//! Zero-copy interop between CoreVideo and wgpu: decoded video frames and
//! encoder pixel buffers are IOSurface-backed, so they can be used as
//! Metal textures directly instead of being copied through the CPU.

use std::collections::HashMap;
use std::ptr::NonNull;
use std::sync::Mutex;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_core_foundation::CFRetained;
use objc2_core_video::*;
use objc2_metal::{MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureType, MTLTextureUsage};

use crate::util::SendBox;

/// Most textures kept for recycled IOSurfaces (decoder and encoder pools
/// hold a handful each); the oldest goes first.
const MAX_SURFACES: usize = 16;

pub struct TextureCache {
    cache: SendBox<CFRetained<CVMetalTextureCache>>,
    device: wgpu::Device,
    mtl: SendBox<Retained<ProtocolObject<dyn objc2_metal::MTLDevice>>>,
    /// A texture per IOSurface (by id and usage), reused when the pixel
    /// buffer pool hands that surface out again. AMD's Metal driver maps
    /// an IOSurface into the GPU with a slow kernel call for every new
    /// texture over it: per frame, that took over half the preview's time.
    /// Only on GPUs with their own memory (None elsewhere: Apple silicon
    /// keeps CoreVideo's texture cache).
    surfaces: Option<Mutex<(HashMap<(u32, u32), wgpu::Texture>, std::collections::VecDeque<(u32, u32)>)>>,
}

/// A pixel buffer viewed as a wgpu texture. Keeps the buffer alive.
pub struct Wrapped {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    _cv: Option<SendBox<CFRetained<CVMetalTexture>>>,
    pub pb: SendBox<CFRetained<CVPixelBuffer>>,
}

impl TextureCache {
    /// None if the device isn't Metal (then callers fall back to copies).
    pub fn new(device: &wgpu::Device) -> Option<Self> {
        let mtl: Retained<ProtocolObject<dyn objc2_metal::MTLDevice>> = {
            let hal = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }?;
            hal.raw_device().clone()
        };
        let mut out = std::ptr::null_mut();
        let st = unsafe { CVMetalTextureCache::create(None, None, &mtl, None, NonNull::from(&mut out)) };
        if st != 0 || out.is_null() {
            return None;
        }
        let cache = unsafe { CFRetained::from_raw(NonNull::new(out)?) };
        let own_memory = !mtl.hasUnifiedMemory();
        let surfaces = own_memory.then(|| Mutex::new((HashMap::new(), Default::default())));
        Some(TextureCache { cache: SendBox(cache), device: device.clone(), mtl: SendBox(mtl), surfaces })
    }

    /// Wrap a BGRA pixel buffer as a texture with the given usage.
    pub fn wrap(&self, pb: &CVPixelBuffer, usage: wgpu::TextureUsages) -> Option<Wrapped> {
        if let Some(w) = self.wrap_surface(pb, usage) {
            return Some(w);
        }
        let (w, h) = (CVPixelBufferGetWidth(pb), CVPixelBufferGetHeight(pb));
        let mut tex = std::ptr::null_mut();
        let st = unsafe {
            CVMetalTextureCache::create_texture_from_image(
                None,
                &self.cache.0,
                pb,
                None,
                MTLPixelFormat::BGRA8Unorm,
                w,
                h,
                0,
                NonNull::from(&mut tex),
            )
        };
        if st != 0 || tex.is_null() {
            return None;
        }
        let cv = unsafe { CFRetained::from_raw(NonNull::new(tex)?) };
        let mtl: Retained<ProtocolObject<dyn MTLTexture>> = CVMetalTextureGetTexture(&cv)?;
        let size = wgpu::Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 };
        let hal = unsafe {
            wgpu::hal::metal::Device::texture_from_raw(
                mtl,
                wgpu::TextureFormat::Bgra8Unorm,
                MTLTextureType::Type2D,
                1,
                1,
                wgpu::hal::CopyExtent { width: w as u32, height: h as u32, depth: 1 },
                None,
            )
        };
        let texture = unsafe {
            self.device.create_texture_from_hal::<wgpu::hal::api::Metal>(
                hal,
                &wgpu::TextureDescriptor {
                    label: Some("corevideo"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    usage,
                    view_formats: &[],
                },
                wgpu::TextureUses::UNINITIALIZED,
            )
        };
        let view = texture.create_view(&Default::default());
        let pb = unsafe { CFRetained::retain(NonNull::from(pb)) };
        Some(Wrapped { texture, view, size: (w as u32, h as u32), _cv: Some(SendBox(cv)), pb: SendBox(pb) })
    }

    /// The texture over `pb`'s IOSurface, made once per surface.
    fn wrap_surface(&self, pb: &CVPixelBuffer, usage: wgpu::TextureUsages) -> Option<Wrapped> {
        if CVPixelBufferGetPixelFormatType(pb) != crate::util::PIX_BGRA {
            return None;
        }
        let surfaces = self.surfaces.as_ref()?;
        let surface = CVPixelBufferGetIOSurface(Some(pb))?;
        let (w, h) = (CVPixelBufferGetWidth(pb) as u32, CVPixelBufferGetHeight(pb) as u32);
        let key = (surface.id(), usage.bits());
        let mut surfaces = surfaces.lock().unwrap_or_else(|e| e.into_inner());
        let (map, order) = &mut *surfaces;
        let texture = match map.get(&key).filter(|t| (t.width(), t.height()) == (w, h)) {
            Some(t) => t.clone(),
            None => {
                // SAFETY: a BGRA texture over the whole IOSurface, which the
                // texture keeps alive.
                let mtl = unsafe {
                    let desc = MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
                        MTLPixelFormat::BGRA8Unorm,
                        w as usize,
                        h as usize,
                        false,
                    );
                    let mut mu = MTLTextureUsage::ShaderRead;
                    if usage.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
                        mu |= MTLTextureUsage::RenderTarget;
                    }
                    if usage.contains(wgpu::TextureUsages::STORAGE_BINDING) {
                        mu |= MTLTextureUsage::ShaderWrite;
                    }
                    desc.setUsage(mu);
                    desc.setStorageMode(MTLStorageMode::Managed);
                    self.mtl.0.newTextureWithDescriptor_iosurface_plane(&desc, &surface, 0)?
                };
                let texture = self.from_metal(mtl, w, h, usage);
                if map.insert(key, texture.clone()).is_none() {
                    order.push_back(key);
                }
                while order.len() > MAX_SURFACES {
                    if let Some(old) = order.pop_front() {
                        map.remove(&old);
                    }
                }
                texture
            }
        };
        drop(surfaces);
        let view = texture.create_view(&Default::default());
        let pb = unsafe { CFRetained::retain(NonNull::from(pb)) };
        Some(Wrapped { texture, view, size: (w, h), _cv: None, pb: SendBox(pb) })
    }

    fn from_metal(&self, mtl: Retained<ProtocolObject<dyn MTLTexture>>, w: u32, h: u32, usage: wgpu::TextureUsages) -> wgpu::Texture {
        let size = wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 };
        // SAFETY: a 2D BGRA texture of exactly this size and usage.
        unsafe {
            let hal = wgpu::hal::metal::Device::texture_from_raw(
                mtl,
                wgpu::TextureFormat::Bgra8Unorm,
                MTLTextureType::Type2D,
                1,
                1,
                wgpu::hal::CopyExtent { width: w, height: h, depth: 1 },
                None,
            );
            self.device.create_texture_from_hal::<wgpu::hal::api::Metal>(
                hal,
                &wgpu::TextureDescriptor {
                    label: Some("iosurface"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    usage,
                    view_formats: &[],
                },
                wgpu::TextureUses::UNINITIALIZED,
            )
        }
    }

    /// Release cached textures no longer in use.
    pub fn flush(&self) {
        self.cache.0.flush(0);
    }
}
