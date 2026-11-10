//! Turns a project + recording into a description of each output frame.
//! The renderer draws a [`Frame`]; nothing here touches the GPU.

use std::collections::HashMap;

use super::animator::{Animator, Target};
use super::camera;
use super::captions::{self, Line};
use super::cursor::{self, ClickFx, CursorModel};
use super::events::Events;
use super::geom::{Rect, Vec2, v2};
use super::keys::{self, Badge};
use super::sizes::Sizes;
use super::spring::SpringConfig;
use super::timemap::TimeMap;
use super::zoom;
use crate::project::model::{CalloutKind, Card, ClickEffect, Color, CursorSkin, LayoutKind, MaskKind, Project, Shadow};
use crate::project::recording::Recording;
use crate::render::art;

/// Directional or radial blur for a moving layer.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Blur {
    /// Streak in output pixels (move blur).
    pub velocity: Vec2,
    /// Radial blur strength (zoom blur) and its centre in output pixels.
    pub zoom: f64,
    pub zoom_center: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowLayer {
    /// Outer window rect (including inset) in output pixels.
    pub rect: Rect,
    pub radius: f64,
    /// Inset thickness in output pixels.
    pub inset: f64,
    pub inset_color: Color,
    pub inset_alpha: f64,
    /// Inner (screen) corner radius.
    pub inner_radius: f64,
    /// Crop of the screen video, normalised.
    pub uv: Rect,
    pub shadow: Shadow,
    /// Output pixels per 1080p-frame pixel times the zoom (for shadow sizing).
    pub scale: f64,
    pub blur: Blur,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CursorLayer {
    /// Hotspot position in output pixels.
    pub pos: Vec2,
    /// Sprite height in output pixels (before press scale).
    pub height: f64,
    pub rotation: f64,
    pub alpha: f64,
    pub scale: f64,
    pub blur: f64,
    pub image: u32,
    pub velocity: Vec2,
}

/// The hand skin's sleeve: a line from the wrist that runs past the frame
/// edge, so the frame cuts it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArmLayer {
    /// The wrist, in output pixels.
    pub from: Vec2,
    /// A point past the frame edge along the fixed diagonal.
    pub to: Vec2,
    /// Sleeve thickness in output pixels.
    pub width: f64,
    pub alpha: f64,
}

/// Fixed sleeve angle, degrees below horizontal, down and to the right.
const ARM_DEG: f64 = 48.0;
/// Where the wrist sits from the fingertip, in hand heights (x right, y down).
const ARM_WRIST: Vec2 = v2(0.33, 0.95);
/// Sleeve thickness in hand heights.
const ARM_WIDTH: f64 = 0.72;

#[derive(Clone, Debug, PartialEq)]
pub struct CircleFx {
    pub pos: Vec2,
    pub radius: f64,
    pub alpha: f64,
    pub blur: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RippleFx {
    pub center: Vec2,
    pub progress: f64,
    pub strength: f64,
    /// Ring radius at the end of the effect, output pixels.
    pub reach: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CameraLayer {
    pub rect: Rect,
    pub radius: f64,
    /// Superellipse exponent of the outline (0 = rounded rectangle).
    pub squircle: f64,
    /// Border width in output pixels and its colour.
    pub border: f64,
    pub border_color: [f32; 4],
    pub shadow: bool,
    pub alpha: f64,
    pub blur: f64,
    pub mirror: bool,
    /// Centre crop of the camera video to the bubble aspect, normalised.
    pub uv: Rect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayer {
    pub text: String,
    /// Font size in output pixels.
    pub size: f64,
    /// Baseline centre position (bottom-centred).
    pub center: Vec2,
    pub alpha: f64,
    pub blur: f64,
    /// Draw as a key badge (pill) instead of a caption.
    pub badge: bool,
    /// Draw the dark backdrop behind the text.
    pub bg: bool,
    /// Corner radius of that backdrop, output px (0 = the default).
    pub corner: f64,
    /// Tint (straight alpha; white leaves the text as drawn).
    pub color: [f32; 4],
    /// `center` is the left edge of the first glyph rather than the middle.
    pub left: bool,
    /// A left-to-right wipe: 0 = nothing shown, 1 = the whole word.
    pub reveal: Option<f64>,
}

/// A region effect on the screen (blur, pixelate or highlight).
#[derive(Clone, Debug, PartialEq)]
pub struct MaskLayer {
    pub kind: MaskKind,
    /// Region in output px (already clipped to the visible screen).
    pub rect: Rect,
    /// The same region in screen-texture uv.
    pub uv: Rect,
    pub radius: f64,
    /// Blur/pixel size in output px, or dim opacity for highlights.
    pub amount: f64,
    pub alpha: f64,
    /// The window's inner (screen) rect and corner radius, for clipping.
    pub screen: Rect,
    pub screen_radius: f64,
}

/// An arrow, box or numbered pin over the screen, in output px.
#[derive(Clone, Debug, PartialEq)]
pub struct CalloutLayer {
    pub kind: CalloutKind,
    /// Bounding rect (box/pin) or arrow span, output px.
    pub rect: Rect,
    /// Arrow start and tip.
    pub from: Vec2,
    pub to: Vec2,
    pub color: Color,
    pub width: f64,
    pub alpha: f64,
}

/// The spinning daisy of the end credit: centre, height and angle in output px.
#[derive(Clone, Debug, PartialEq)]
pub struct DaisyLayer {
    pub center: Vec2,
    pub size: f64,
    pub angle: f64,
    pub alpha: f64,
    /// Output px it moved in the last frame: the streak it leaves.
    pub velocity: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// Output size in pixels.
    pub size: Vec2,
    pub playback_ms: f64,
    pub source_ms: f64,
    /// Extra background/screen blur (camera-only layout), output pixels.
    pub backdrop_blur: f64,
    pub window: WindowLayer,
    pub cursor: Option<CursorLayer>,
    /// The hand skin's sleeve, present with the cursor and nulled where it is.
    pub arm: Option<ArmLayer>,
    pub circles: Vec<CircleFx>,
    pub ripples: Vec<RippleFx>,
    pub camera: Option<CameraLayer>,
    pub camera_full: Option<CameraLayer>,
    pub texts: Vec<TextLayer>,
    pub masks: Vec<MaskLayer>,
    pub callouts: Vec<CalloutLayer>,
    /// Overall window opacity (dips to 0 at a cut when transitions are on).
    pub window_alpha: f64,
    /// Rotation of the daisy mark, radians (0 when resting).
    pub watermark_spin: f64,
    /// True on an intro/outro card: the background and text only, no window.
    pub card: bool,
    /// True on the app's credit card (dark, with the daisy).
    pub credit: bool,
    /// The credit's daisy, when on that card.
    pub daisy: Option<DaisyLayer>,
    /// Device frame shapes around the window, back to front.
    pub device: Vec<super::device::Part>,
    /// The device body outline for the drop shadow.
    pub device_body: Option<(Rect, f64)>,
}

struct Anim {
    screen: Animator,
    cursor: Animator,
    fade: Animator,
    bubble: Animator,
    bubble_vis: Animator,
    full: Animator,
    backdrop: Animator,
    badges: HashMap<usize, Animator>,
}

impl Anim {
    fn new(fps: f64) -> Self {
        Anim {
            screen: Animator::new(fps),
            cursor: Animator::new(fps),
            fade: Animator::new(fps),
            bubble: Animator::new(fps),
            bubble_vis: Animator::new(fps),
            full: Animator::new(fps),
            backdrop: Animator::new(fps),
            badges: HashMap::new(),
        }
    }
}

pub struct Scene {
    pub project: Project,
    pub sizes: Sizes,
    pub map: TimeMap,
    events: Events,
    badges: Vec<Badge>,
    lines: Vec<Line>,
    camera_video: Option<Vec2>,
    has_cursor: bool,
    duration: f64,
    fps: f64,
    anim: Anim,
}

impl Scene {
    pub fn new(project: Project, rec: &Recording, fps: f64) -> Self {
        let (w, h) = rec.screen_size();
        let sizes = Sizes::new(&project.style, v2(w, h));
        let map = super::timemap::for_project(&project, rec.meta.duration_ms);
        Scene {
            sizes,
            map,
            events: Events::new(rec.mouse.clone()),
            badges: keys::badges(&rec.keys, project.style.shortcuts.single_keys),
            lines: super::words::caption_lines(&rec.transcript, &project.timeline.slices, rec.meta.duration_ms),
            camera_video: rec.meta.camera.as_ref().map(|c| v2(c.width as f64, c.height as f64)),
            has_cursor: !rec.mouse.is_empty(),
            duration: rec.meta.duration_ms,
            project,
            fps,
            anim: Anim::new(fps),
        }
    }

    /// Replace the project (after an edit) and drop cached animation state.
    pub fn set_project(&mut self, project: Project, rec: &Recording) {
        let fps = self.fps;
        *self = Scene::new(project, rec, fps);
    }

    pub fn fps(&self) -> f64 {
        self.fps
    }

    pub fn duration(&self) -> f64 {
        self.map.duration()
    }

    pub fn frame_count(&self) -> u64 {
        (self.duration() * self.fps / 1000.0).ceil() as u64
    }

    pub fn frame_index(&self, playback_ms: f64) -> i64 {
        (playback_ms * self.fps / 1000.0).round() as i64
    }

    fn screen_values(&mut self, f: i64) -> Vec<f64> {
        let (map, sizes, tl, events) = (&self.map, &self.sizes, &self.project.timeline, &self.events);
        let spring = self.project.style.screen_spring;
        let cm = CursorModel {
            events,
            style: &self.project.style.cursor,
            slices: &tl.slices,
            source_start: map.source_start(),
            source_end: map.source_end().min(self.duration),
        };
        self.anim.screen.values_at(f, |pb| {
            let src = map.to_source(pb);
            let focus = zoom::focus_at(tl, events, sizes, src, cm.target(src));
            let (scale, off) = zoom::place(sizes, focus);
            Target { values: vec![scale, off.x, off.y], spring, snap: zoom::is_instant_edge(&tl.zooms, src) }
        })
    }

    fn cursor_values(&mut self, f: i64) -> Vec<f64> {
        let map = &self.map;
        let cm = CursorModel {
            events: &self.events,
            style: &self.project.style.cursor,
            slices: &self.project.timeline.slices,
            source_start: map.source_start(),
            source_end: map.source_end().min(self.duration),
        };
        self.anim.cursor.values_at(f, |pb| {
            let src = map.to_source(pb);
            let p = cm.target(src).unwrap_or_default();
            Target::new(vec![p.x, p.y, cm.tilt(src)], cm.spring(src))
        })
    }

    /// Window rect in output pixels for a frame, plus its zoom scale.
    fn window_rect(&mut self, f: i64, k: f64) -> (Rect, f64) {        let v = self.screen_values(f);
        let body = self.sizes.default_body();
        let pad = Vec2::splat(self.sizes.pad);
        let min = pad + body.min() + v2(v[1], v[2]);
        (Rect::from_min_size(min, body.size() * v[0]).scale_about_origin(k), v[0])
    }

    /// Map an output-pixel point back to screen-video pixels given the window rect.
    pub fn output_to_screen(&self, p: Vec2, window: Rect) -> Vec2 {        let s = &self.sizes;
        let local = (p - window.min()).div(window.size().div(s.window));
        local + s.crop.min() - Vec2::splat(s.inset)
    }

    /// Map a screen-video point to output pixels given the window rect.
    pub fn screen_to_output(&self, p: Vec2, window: Rect) -> Vec2 {
        let s = &self.sizes;
        let local = p - s.crop.min() + Vec2::splat(s.inset);
        window.min() + local.mul(window.size().div(s.window))
    }

    /// Shift a window rect so the action stays in frame (a forced shape only).
    fn reframed_rect(&self, rect: Rect, src: f64, size: Vec2) -> Rect {
        let style = &self.project.style;
        if !style.auto_reframe || style.aspect.ratio().is_none() {
            return rect;
        }
        let cm = CursorModel {
            events: &self.events,
            style: &style.cursor,
            slices: &self.project.timeline.slices,
            source_start: self.map.source_start(),
            source_end: self.map.source_end().min(self.duration),
        };
        let focus = zoom::focus_at(&self.project.timeline, &self.events, &self.sizes, src, cm.target(src)).point;
        let focus_out = self.screen_to_output(focus, rect);
        rect.translate(self.sizes.reframe_pan(rect, size, focus_out))
    }
    /// Describe the frame at a playback time for an output of `height` px.
    pub fn frame(&mut self, playback_ms: f64, height: f64) -> Frame {
        let f = self.frame_index(playback_ms);
        let pb = f as f64 * 1000.0 / self.fps;
        let src = self.map.to_source(pb);
        let k = height / self.sizes.output.y;
        let size = self.sizes.output * k;
        let style = self.project.style.clone();
        let blur_gain = self.fps / 60.0;

        // The daisy mark turns slowly every few seconds, like the splash.
        let watermark_spin = {
            const PERIOD_MS: f64 = 4000.0;
            const SPIN_MS: f64 = 1200.0;
            let t = pb.rem_euclid(PERIOD_MS);
            if t < SPIN_MS {
                let k = t / SPIN_MS;
                k * k * (3.0 - 2.0 * k) * std::f64::consts::TAU
            } else {
                0.0
            }
        };

        // Window and its motion.
        let (rect0, zoom_scale) = self.window_rect(f, k);
        // With a forced shape, follow the action so it stays in frame.
        let rect = self.reframed_rect(rect0, src, size);
        let prev = if f > 0 {
            let prev_src = self.map.to_source((f - 1) as f64 * 1000.0 / self.fps);
            let (pr, _) = self.window_rect(f - 1, k);
            Some(self.reframed_rect(pr, prev_src, size))
        } else {
            None
        };
        let mut wblur = Blur::default();
        if let Some(prev) = prev {
            let mb = &style.motion_blur;
            let d_center = rect.center() - prev.center();
            let d_size = (rect.size() - prev.size()).length();
            if d_size > d_center.length() && d_size >= 1.0 {
                let ratio = rect.diagonal() / prev.diagonal();
                wblur.zoom = (1.0 - ratio).abs() * mb.amount * mb.screen_zoom * blur_gain;
                if (prev.w - rect.w).abs() > 1e-6 {
                    wblur.zoom_center = (rect.min() * prev.w - prev.min() * rect.w) / (prev.w - rect.w);
                }
            } else if d_center.length() >= 1.0 {
                wblur.velocity = d_center * (mb.amount * mb.screen_move * blur_gain);
            }
        }
        // Roundness and shadow are px in a 1080p-tall frame at rest, whatever
        // the recording's resolution (like the background blur), growing
        // with the zoom.
        let style_px = k * self.sizes.output.y / 1080.0 * zoom_scale / self.sizes.initial_zoom;
        let rscale = rect.w / self.sizes.window.x;
        let inset = self.sizes.inset * rscale;
        let radius = style.corner_radius * style_px;
        let inner_radius = (radius - inset).max(4.0 * style_px);
        let rec = self.sizes.recording;
        let crop = self.sizes.crop;
        let device_screen = super::device::screen_radius(style.device, rect);
        let radius = device_screen.unwrap_or(radius);
        let inner_radius = device_screen.map_or(inner_radius, |r| (r - inset).max(0.0));
        let window = WindowLayer {
            rect,
            radius,
            inset,
            inset_color: style.inset.color,
            inset_alpha: style.inset.alpha,
            inner_radius: if inset > 0.0 { inner_radius } else { radius },
            uv: Rect::new(crop.x / rec.x, crop.y / rec.y, crop.w / rec.x, crop.h / rec.y),
            shadow: style.shadow,
            scale: style_px,
            blur: wblur,
        };

        // The visible (cropped) screen in output px.
        let screen_rect = Rect::from_min_size(
            self.screen_to_output(self.sizes.crop.min(), rect),
            self.sizes.crop.size().mul(rect.size().div(self.sizes.window)),
        );

        // Cursor.
        let mut cursor = None;
        let mut arm = None;
        let mut circles = vec![];
        let mut ripples = vec![];
        if self.has_cursor && !style.cursor.hide {
            let cv = self.cursor_values(f);
            let pv = if f > 0 { Some(self.cursor_values(f - 1)) } else { None };
            let map = &self.map;
            let cm = CursorModel {
                events: &self.events,
                style: &self.project.style.cursor,
                slices: &self.project.timeline.slices,
                source_start: map.source_start(),
                source_end: map.source_end().min(self.duration),
            };
            let fv = self.anim.fade.values_at(f, |pb| Target::new(cm.fade(map.to_source(pb)).to_vec(), SpringConfig::PRESS));
            let pos = self.screen_to_output(v2(cv[0], cv[1]), rect);
            let velocity = match (&pv, prev) {
                (Some(p), Some(prev)) => {
                    let before = self.screen_to_output(v2(p[0], p[1]), prev);
                    (pos - before) * (style.motion_blur.amount * style.motion_blur.cursor * blur_gain)
                }
                _ => Vec2::ZERO,
            };
            let image = Events::last_at(&self.events.all, src).map_or(0, |e| e.cursor);
            // A cursor outside the cropped screen isn't shown.
            let visible = screen_rect.inset(-2.0).contains(pos);
            let vis_alpha = if visible { fv[0].clamp(0.0, 1.0) } else { 0.0 };
            cursor = Some(CursorLayer {
                pos,
                height: cursor::BASE_HEIGHT * style.cursor.size * k * zoom_scale,
                rotation: cv[2],
                alpha: vis_alpha,
                scale: fv[1],
                blur: fv[2].max(0.0),
                image,
                velocity,
            });
            if style.cursor.skin == CursorSkin::Hand {
                // The sleeve: steady, from the wrist past the frame edge, so
                // the frame always cuts it. It fades with the hand.
                // 20 sprite units is the hand's content height, so this is the
                // hand as drawn, and the sleeve follows it.
                let hand = cursor::BASE_HEIGHT * style.cursor.size * k * zoom_scale * 20.0 / art::HAND_UNITS;
                let rot = cv[2].to_radians();
                let off = ARM_WRIST * hand;
                let wrist = pos + v2(off.x * rot.cos() - off.y * rot.sin(), off.x * rot.sin() + off.y * rot.cos());
                let a = (ARM_DEG + style.cursor.base_rotation).to_radians();
                let dir = v2(a.cos(), a.sin());
                arm = Some(ArmLayer { from: wrist, to: wrist + dir * (size.x + size.y), width: ARM_WIDTH * hand, alpha: vis_alpha });
            }
            let fx_px = 16.0 * style.cursor.size * k * zoom_scale;
            match style.cursor.click_effect {
                ClickEffect::Circle => {
                    circles = cursor::circle_effects(&self.events, src)
                        .into_iter()
                        .map(|c: ClickFx| CircleFx {
                            pos: self.screen_to_output(c.pos, rect),
                            radius: fx_px * c.scale,
                            alpha: c.alpha * 0.6,
                            blur: c.blur * k,
                        })
                        .collect();
                }
                ClickEffect::Ripple { strength } => {
                    ripples = cursor::ripple_effects(&self.events, src)
                        .into_iter()
                        .map(|c| RippleFx {
                            center: self.screen_to_output(c.pos, rect),
                            progress: c.progress,
                            strength,
                            reach: 400.0 * k * zoom_scale,
                        })
                        .collect();
                }
                ClickEffect::None => {}
            }
        }

        // Camera.
        let layout = camera::kind_at(&self.project.timeline.layouts, src);
        let map = self.map.clone();
        let layouts = self.project.timeline.layouts.clone();
        let backdrop = self.anim.backdrop.values_at(f, |pb| {
            let only_cam = camera::kind_at(&layouts, map.to_source(pb)) == LayoutKind::CameraOnly;
            Target::new(vec![if only_cam { 15.0 } else { 0.0 }], SpringConfig::PRESS)
        })[0];
        let (mut cam, mut cam_full) = (None, None);
        if let Some(video) = self.camera_video.filter(|_| !style.camera.hide) {
            let cs = style.camera;
            let zooms = self.project.timeline.zooms.clone();
            let out = self.sizes.output;
            let b = self.anim.bubble.values_at(f, |pb| {
                let s = map.to_source(pb);
                let zoomed = zoom::active(&zooms, s).is_some();
                let r = camera::bubble_at(&cs, &layouts, out, video, s, zoomed);
                Target { values: vec![r.x, r.y, r.w, r.h], spring: style.screen_spring, snap: zoom::is_instant_edge(&zooms, s) }
            });
            let vis = self.anim.bubble_vis.values_at(f, |pb| {
                let shown = camera::kind_at(&layouts, map.to_source(pb)) == LayoutKind::Both;
                Target::new(if shown { vec![1.0, 0.0] } else { vec![0.0, 10.0] }, SpringConfig::FADE)
            });
            let r = Rect::new(b[0], b[1], b[2], b[3]).scale_about_origin(k);
            let aspect = camera::aspect(&cs, video);
            let uv = centre_crop(video.aspect(), aspect);
            if vis[0] > 0.002 {
                cam = Some(CameraLayer {
                    rect: r,
                    radius: cs.radius(r.w, r.h),
                    squircle: cs.squircle_exponent(),
                    border: cs.border_px(size.y),
                    border_color: cs.border_color.to_f32(),
                    shadow: cs.shadow,
                    alpha: vis[0].clamp(0.0, 1.0),
                    blur: vis[1].max(0.0) * k,
                    mirror: cs.mirror,
                    uv,
                });
            }
            let full = self.anim.full.values_at(f, |pb| {
                let on = camera::kind_at(&layouts, map.to_source(pb)) == LayoutKind::CameraOnly;
                Target::new(if on { vec![1.0, 0.0, 1.0] } else { vec![0.0, 30.0, 1.05] }, SpringConfig::FADE)
            });
            if full[0] > 0.002 {
                let fr = Rect::from_min_size(Vec2::ZERO, size);
                let grown = Rect::from_min_size(fr.center() - size * (full[2] / 2.0), size * full[2]);
                cam_full = Some(CameraLayer {
                    rect: grown,
                    radius: 0.0,
                    squircle: 0.0,
                    border: 0.0,
                    border_color: [0.0; 4],
                    shadow: false,
                    alpha: full[0].clamp(0.0, 1.0),
                    blur: full[1].max(0.0) * k,
                    mirror: cs.mirror,
                    uv: centre_crop(video.aspect(), size.aspect()),
                });
            }
        }
        let _ = layout;

        // Overlays.
        let mut texts = vec![];
        let base_font = (size.y * 0.05).min(size.x * 0.07);
        if style.captions.show
            && let Some(line) = captions::line_at(&self.lines, src)
        {
            let c = &style.captions;
            let text = if c.uppercase { line.text().to_uppercase() } else { line.text() };
            texts.push(TextLayer {
                size: fit_font(&text, base_font * c.size, size.x * 0.92),
                text,
                center: v2(size.x / 2.0, size.y * (0.94 - 0.88 * c.place)),
                alpha: 1.0,
                blur: 0.0,
                badge: false,
                bg: c.backdrop,
                corner: c.radius * k,
                left: false,
                reveal: None,
                color: c.color.to_f32(),
            });
        }
        if style.shortcuts.show {
            let font = base_font * style.shortcuts.size;
            let caption_lift = if texts.is_empty() { 0.0 } else { base_font * style.captions.size * 1.8 };
            let near: Vec<usize> = (0..self.badges.len())
                .filter(|&i| {
                    let b = &self.badges[i];
                    src >= b.start - 600.0 && src <= b.end + 1500.0
                })
                .collect();
            for i in near {
                let b = self.badges[i].clone();
                let map = self.map.clone();
                let anim = self.anim.badges.entry(i).or_insert_with(|| Animator::new(self.fps));
                let lift = font * 0.6;
                let v = anim.values_at(f, |pb| {
                    let s = map.to_source(pb);
                    let vals = if s < b.start {
                        vec![0.0, lift, 6.0]
                    } else if s <= b.end {
                        vec![1.0, 0.0, 0.0]
                    } else {
                        vec![0.0, -lift, 6.0]
                    };
                    Target::new(vals, SpringConfig::BADGE)
                });
                if v[0] > 0.01 {
                    let label = if b.count > 1 { format!("{}  ×{}", b.label, b.count) } else { b.label.clone() };
                    texts.push(TextLayer {
                        text: label,
                        size: font,
                        center: v2(size.x / 2.0, size.y - font * 1.8 - caption_lift + v[1]),
                        alpha: v[0].clamp(0.0, 1.0),
                        blur: v[2] * k,
                        badge: true,
                bg: true,
                corner: 0.0,
                left: false,
                reveal: None,
                color: [1.0; 4],
                    });
                }
            }
        }

        // Masks, clipped to the visible (cropped) screen.
        let zk = k * zoom_scale;
        let masks = self
            .project
            .timeline
            .masks
            .iter()
            .flat_map(|m| {
                let mut out_layers = vec![];
                let alpha = m.alpha_at(src, self.duration);
                if alpha <= 0.002 {
                    return out_layers;
                }
                let amount = if m.kind == MaskKind::Highlight { m.amount } else { m.amount * zk };
                let mut push = |rec_px: Rect, radius: f64| {
                    let out = Rect::from_points(self.screen_to_output(rec_px.min(), rect), self.screen_to_output(rec_px.max(), rect));
                    out_layers.push(MaskLayer {
                        kind: if m.kind == MaskKind::Brush { MaskKind::Blur } else { m.kind },
                        rect: out,
                        uv: Rect::new(rec_px.x / rec.x, rec_px.y / rec.y, rec_px.w / rec.x, rec_px.h / rec.y),
                        radius,
                        amount,
                        alpha,
                        screen: screen_rect,
                        screen_radius: window.inner_radius,
                    });
                };
                if m.kind == MaskKind::Brush {
                    // One soft circle per dab: a painted stroke.
                    for d in &m.dabs {
                        let r = d[2].max(0.004) as f64;
                        let (dx, dy) = (d[0] as f64, d[1] as f64);
                        // A circle: the same pixel radius both ways.
                        let side = 2.0 * r * rec.x;
                        let rec_px = Rect::new(dx * rec.x - side / 2.0, dy * rec.y - r * rec.x, side, side).intersect(&self.sizes.crop);
                        if rec_px.w <= 0.5 || rec_px.h <= 0.5 {
                            continue;
                        }
                        let out = Rect::from_points(self.screen_to_output(rec_px.min(), rect), self.screen_to_output(rec_px.max(), rect));
                        push(rec_px, out.w.min(out.h) * 0.5);
                    }
                } else {
                    let rec_px = Rect::new(m.rect.x * rec.x, m.rect.y * rec.y, m.rect.w * rec.x, m.rect.h * rec.y).intersect(&self.sizes.crop);
                    if rec_px.w > 0.0 && rec_px.h > 0.0 {
                        push(rec_px, m.radius * zk);
                    }
                }
                out_layers
            })
            .collect();

        // Callouts: pinned in the recording, following the window and its zoom.
        let mut callouts = vec![];
        for c in &self.project.timeline.callouts {
            let alpha = c.alpha_at(src, self.duration);
            if alpha <= 0.002 {
                continue;
            }
            let from = self.screen_to_output(v2(c.rect.x * rec.x, c.rect.y * rec.y), rect);
            let to = self.screen_to_output(v2((c.rect.x + c.rect.w) * rec.x, (c.rect.y + c.rect.h) * rec.y), rect);
            callouts.push(CalloutLayer {
                kind: c.kind,
                rect: Rect::from_points(from, to),
                from,
                to,
                color: c.color,
                width: c.width * zk,
                alpha,
            });
            // A step's number, or an arrow/box label, drawn as plain text.
            if c.kind == CalloutKind::Step {
                texts.push(TextLayer {
                    text: c.step.to_string(),
                    size: c.width * zk * 2.4,
                    center: from,
                    alpha,
                    blur: 0.0,
                    badge: false,
                    bg: false,
                    corner: 0.0,
                    left: false,
                    reveal: None,
                    color: [1.0; 4],
                });
            } else if !c.text.trim().is_empty() {
                texts.push(TextLayer {
                    text: c.text.clone(),
                    size: (c.width * zk * 2.2).max(12.0),
                    center: (from + to) / 2.0,
                    alpha,
                    blur: 0.0,
                    badge: false,
                    bg: true,
                    corner: 0.0,
                    left: false,
                    reveal: None,
                    color: [1.0; 4],
                });
            }
        }

        let mut frame = Frame {
            size,
            playback_ms: pb,
            source_ms: src,
            backdrop_blur: backdrop * k,
            window,
            cursor,
            arm,
            circles,
            ripples,
            camera: cam,
            camera_full: cam_full,
            texts,
            masks,
            callouts,
            window_alpha: 1.0,
            watermark_spin,
            card: false,
            credit: false,
            daisy: None,
            device: super::device::parts(style.device, rect),
            device_body: super::device::body(style.device, rect),
        };
        // The app's credit: a dark card where the daisy spins, then slides
        // aside with a streak of motion, and "Made in DaisyCap" wipes in from
        // its edge as the slide lands, rising a little and coming into focus.
        if self.map.on_credit(pb) {
            let t = pb - self.map.credit_start();
            let ms = self.map.credit_ms;
            let ease = |k: f64| {
                let k = k.clamp(0.0, 1.0);
                1.0 - (1.0 - k).powi(3)
            };
            let fade_in = (t / 220.0).clamp(0.0, 1.0);
            let fade_out = ((ms - t) / 320.0).clamp(0.0, 1.0);
            let alpha = (fade_in * fade_out).clamp(0.0, 1.0);
            // The card's beats are fractions of its length, so a short card still
            // shows the word and a long one holds it.
            let at = |f: f64| f * ms;
            let spin = ease((t - at(0.035)) / (at(0.418) - at(0.035))) * 2.0 * std::f64::consts::TAU;
            let size_px = (size.y * 0.16).max(24.0);
            let gap = size_px * 0.35;
            let font = size_px * 0.36;
            let text = "Made in DaisyCap";
            // Measured with the font the renderer draws with, at the size it draws at.
            let text_w = crate::fonts::line_width(text, font.round().max(4.0) as f32) as f64;
            let travel = (gap + text_w) / 2.0;
            let slide_at = |t: f64| ease((t - at(0.441)) / (at(0.603) - at(0.441)));
            let daisy_x = |t: f64| size.x / 2.0 - slide_at(t) * travel;
            // How far the daisy moved in the last frame: the streak it leaves.
            let step_ms = 1000.0 / self.fps;
            let velocity = v2(daisy_x(t) - daisy_x(t - step_ms), 0.0) * style.motion_blur.amount;
            let reveal = ease((t - at(0.515)) / (at(0.750) - at(0.515)));
            let cy = size.y / 2.0;
            frame.credit = true;
            frame.card = true;
            frame.cursor = None;
            frame.arm = None;
            frame.circles.clear();
            frame.ripples.clear();
            frame.camera = None;
            frame.camera_full = None;
            frame.masks.clear();
            frame.callouts.clear();
            frame.device.clear();
            frame.device_body = None;
            frame.daisy = Some(DaisyLayer { center: v2(daisy_x(t), cy), size: size_px, angle: spin, alpha, velocity });
            frame.texts = if reveal > 0.0 && alpha > 0.002 {
                vec![TextLayer {
                    text: text.into(),
                    size: font,
                    center: v2(daisy_x(t) + size_px / 2.0 + gap, cy + (1.0 - reveal) * size_px * 0.22),
                    alpha,
                    blur: (1.0 - reveal) * 6.0 * k,
                    badge: false,
                    bg: false,
                    corner: 0.0,
                    color: [1.0; 4],
                    left: true,
                    reveal: Some(reveal),
                }]
            } else {
                vec![]
            };
        }

        // A dip to the background where the recording jumps at a cut, when
        // transitions are on. At a plain split the recording is continuous, so
        // the halves join seamlessly and nothing dips.
        const JUMP_MS: f64 = 1.0;
        if style.transition_ms > 0.5 && self.map.links.len() > 1 {
            let half = style.transition_ms;
            for w in self.map.links.windows(2) {
                if w[0].slice != w[1].slice && (w[1].src_start - w[0].src_end).abs() > JUMP_MS {
                    let d = (pb - w[1].pb_start).abs();
                    frame.window_alpha = frame.window_alpha.min((d / half).clamp(0.0, 1.0));
                }
            }
            if frame.window_alpha < 0.999 {
                let a = frame.window_alpha;
                if let Some(c) = &mut frame.cursor {
                    c.alpha *= a;
                }
                if let Some(arm) = &mut frame.arm {
                    arm.alpha *= a;
                }
                if let Some(c) = &mut frame.camera {
                    c.alpha *= a;
                }
                if let Some(c) = &mut frame.camera_full {
                    c.alpha *= a;
                }
                for c in &mut frame.callouts {
                    c.alpha *= a;
                }
            }
        }

        // On a card the whole frame is replaced: the project's own background,
        // a title, an optional second line, and the daisy mark. The recording
        // and everything pinned to it (cursor, clicks, camera, masks) is left
        // to the animators above but not drawn, so seeking into a card and back
        // out lands on the same frames as playing straight through.
        if self.map.on_card(pb) {
            let (card, ms, local) = if pb < self.map.intro_ms {
                (style.intro.as_ref(), self.map.intro_ms, pb)
            } else {
                (style.outro.as_ref(), self.map.outro_ms, pb - self.map.recording_end())
            };
            if let Some(card) = card {
                let alpha = Card::alpha(local, ms);
                frame.card = true;
                frame.cursor = None;
                frame.arm = None;
                frame.circles.clear();
                frame.ripples.clear();
                frame.camera = None;
                frame.camera_full = None;
                frame.masks.clear();
                frame.callouts.clear();
                frame.device.clear();
                frame.device_body = None;
                frame.texts = card_texts(card, alpha, size, k);
            }
        }
        frame
    }
}

/// The title and subtitle of a card as text layers, faded by `alpha`.
fn card_texts(card: &Card, alpha: f64, size: Vec2, k: f64) -> Vec<TextLayer> {
    let title_size = (size.y * 0.085).min(size.x * 0.11).max(8.0);
    let sub_size = title_size * 0.42;
    let max_w = size.x * 0.8;
    let mut out = vec![];
    if !card.title.trim().is_empty() {
        let step = if card.subtitle.trim().is_empty() { 0.0 } else { title_size * 0.95 };
        out.push(TextLayer {
            text: card.title.clone(),
            size: fit_font(&card.title, title_size, max_w),
            center: v2(size.x / 2.0, size.y / 2.0 - step * 0.5),
            alpha,
            blur: (8.0 * k * (1.0 - alpha)).max(0.0),
            badge: false,
                bg: true,
                corner: 0.0,
                left: false,
                reveal: None,
                color: [1.0; 4],
        });
    }
    if !card.subtitle.trim().is_empty() {
        out.push(TextLayer {
            text: card.subtitle.clone(),
            size: fit_font(&card.subtitle, sub_size, max_w),
            center: v2(size.x / 2.0, size.y / 2.0 + title_size * 0.85),
            alpha: alpha * 0.85,
            blur: (6.0 * k * (1.0 - alpha)).max(0.0),
            badge: false,
                bg: true,
                corner: 0.0,
                left: false,
                reveal: None,
                color: [1.0; 4],
        });
    }
    out
}

/// Shrink a one-line label's font so it fits `max_w` (narrow outputs).
fn fit_font(text: &str, size: f64, max_w: f64) -> f64 {
    // Inter SemiBold averages ~0.56 em per character, plus the pill padding.
    let est = text.chars().count() as f64 * size * 0.56 + size;
    if est > max_w { size * max_w / est } else { size }
}

/// Normalised centre crop that turns `from` aspect into `to` aspect.
pub fn centre_crop(from: f64, to: f64) -> Rect {
    if from > to {
        let w = to / from;
        Rect::new((1.0 - w) / 2.0, 0.0, w, 1.0)
    } else {
        let h = from / to;
        Rect::new(0.0, (1.0 - h) / 2.0, 1.0, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::model::{Aspect, CursorSkin, Slice, Zoom};
    use crate::project::recording::{MouseEvent, MouseKind, RecordingMeta, VideoTrack};

    fn recording() -> Recording {
        let mut mouse = vec![];
        for i in 0..200 {
            let t = i as f64 * 50.0;
            mouse.push(MouseEvent { t, x: 100.0 + i as f64 * 5.0, y: 300.0, kind: MouseKind::Move, button: 0, cursor: 0 });
        }
        mouse.push(MouseEvent { t: 3000.0, x: 1500.0, y: 800.0, kind: MouseKind::Down, button: 0, cursor: 0 });
        mouse.push(MouseEvent { t: 3080.0, x: 1500.0, y: 800.0, kind: MouseKind::Up, button: 0, cursor: 0 });
        Recording {
            meta: RecordingMeta {
                duration_ms: 10_000.0,
                screen: VideoTrack { file: "screen.mov".into(), width: 1920, height: 1080, fps: 60.0, offset_ms: 0.0 },
                scale: 2.0,
                ..Default::default()
            },
            mouse,
            ..Default::default()
        }
    }

    #[test]
    fn frame_is_deterministic_and_zooms() {
        let rec = recording();
        let mut p = Project::default();
        p.timeline.zooms.push(Zoom { start: 2500.0, end: 6000.0, scale: 2.0, ..Default::default() });
        let mut a = Scene::new(p.clone(), &rec, 60.0);
        let before = a.frame(1000.0, 1080.0);
        let during = a.frame(5000.0, 1080.0);
        assert!(during.window.rect.w > before.window.rect.w * 1.9, "zoomed in");
        // A fresh scene seeking directly gives the same frame.
        let mut b = Scene::new(p, &rec, 60.0);
        assert_eq!(b.frame(5000.0, 1080.0).window, during.window);
    }

    #[test]
    fn zoom_transition_has_blur() {
        let rec = recording();
        let mut p = Project::default();
        p.timeline.zooms.push(Zoom { start: 2500.0, end: 6000.0, scale: 2.0, ..Default::default() });
        let mut s = Scene::new(p, &rec, 60.0);
        let f = s.frame(2600.0, 1080.0);
        assert!(f.window.blur.zoom > 0.0 || f.window.blur.velocity.length() > 0.0);
    }

    #[test]
    fn cursor_inside_window() {
        let rec = recording();
        let mut s = Scene::new(Project::default(), &rec, 60.0);
        let f = s.frame(1000.0, 1080.0);
        let c = f.cursor.unwrap();
        assert!(f.window.rect.contains(c.pos), "{:?} in {:?}", c.pos, f.window.rect);
    }

    #[test]
    fn callouts_follow_and_clear_on_cards() {
        use crate::project::model::Callout;
        let rec = recording();
        let mut p = Project::default();
        p.timeline.callouts.push(Callout { start: 2000.0, end: 6000.0, ..Default::default() });
        let mut s = Scene::new(p.clone(), &rec, 60.0);
        let mid = s.frame(4000.0, 1080.0);
        assert_eq!(mid.callouts.len(), 1, "callout shows during its range");
        assert!(s.frame(8000.0, 1080.0).callouts.is_empty(), "and not after it");

        p.style.intro = Some(Card { title: "Hi".into(), subtitle: String::new(), ms: 2000.0 });
        let mut s = Scene::new(p, &rec, 60.0);
        assert!(s.frame(1000.0, 1080.0).callouts.is_empty(), "cards hide callouts");
        assert_eq!(s.frame(5000.0, 1080.0).callouts.len(), 1, "which return with the recording");
    }

    #[test]
    fn transitions_dip_to_the_background_at_a_cut() {
        let rec = recording();
        let mut p = Project::default();
        p.timeline.slices = vec![
            Slice { id: 1, start: 0.0, end: 3000.0, ..Default::default() },
            Slice { id: 2, start: 5000.0, end: 8000.0, ..Default::default() },
        ];
        p.style.transition_ms = 200.0;
        let mut s = Scene::new(p, &rec, 60.0);
        assert!(s.frame(3000.0, 1080.0).window_alpha < 0.05, "dips at the cut");
        assert!(s.frame(2000.0, 1080.0).window_alpha > 0.9, "opaque away from it");
    }

    #[test]
    fn a_split_stays_seamless_with_transitions_on() {
        let rec = recording();
        let mut p = Project::default();
        // A split: the halves meet with no frames jumped in between.
        p.timeline.slices = vec![
            Slice { id: 1, start: 0.0, end: 3000.0, ..Default::default() },
            Slice { id: 2, start: 3000.0, end: 6000.0, ..Default::default() },
        ];
        p.style.transition_ms = 200.0;
        let mut s = Scene::new(p, &rec, 60.0);
        assert!(s.frame(3000.0, 1080.0).window_alpha > 0.99, "no dip at the split");
        assert!(s.frame(2900.0, 1080.0).window_alpha > 0.99, "nor just before it");
        assert!(s.frame(3100.0, 1080.0).window_alpha > 0.99, "nor just after it");
    }

    fn rec_with_cursor(x: f64, y: f64) -> Recording {
        let mut r = recording();
        r.mouse = vec![MouseEvent { t: 0.0, x, y, kind: MouseKind::Move, button: 0, cursor: 0 }];
        r
    }

    fn hand_project() -> Project {
        let mut p = Project::default();
        p.style.cursor.skin = CursorSkin::Hand;
        p
    }

    #[test]
    fn the_sleeve_always_leaves_the_frame() {
        for (x, y) in [(60.0, 60.0), (1860.0, 60.0), (60.0, 1020.0), (1860.0, 1020.0), (960.0, 540.0)] {
            for aspect in [Aspect::Auto, Aspect::Ratio { w: 9, h: 16 }] {
                let mut p = hand_project();
                p.style.aspect = aspect;
                let mut s = Scene::new(p, &rec_with_cursor(x, y), 60.0);
                let f = s.frame(500.0, 1080.0);
                let arm = f.arm.expect("the hand skin draws an arm");
                assert!(
                    arm.to.x < -arm.width || arm.to.x > f.size.x + arm.width || arm.to.y < -arm.width || arm.to.y > f.size.y + arm.width,
                    "{x},{y} {aspect:?} -> {arm:?}"
                );
            }
        }
        // No arm on the other skins.
        let mut s = Scene::new(Project::default(), &recording(), 60.0);
        assert!(s.frame(500.0, 1080.0).arm.is_none());
    }

    #[test]
    fn the_arm_ignores_tilt_but_follows_base_rotation() {
        let rec = recording();
        let dir = |s: &mut Scene, t: f64| {
            let a = s.frame(t, 1080.0).arm.unwrap();
            let d = a.to - a.from;
            v2(d.x / d.length(), d.y / d.length())
        };
        let mut s = Scene::new(hand_project(), &rec, 60.0);
        let (d0, d1) = (dir(&mut s, 500.0), dir(&mut s, 2500.0));
        assert!((d0 - d1).length() < 1e-6, "a tilt never turns the arm");
        let mut p = hand_project();
        p.style.cursor.base_rotation = 20.0;
        let d2 = dir(&mut Scene::new(p, &rec, 60.0), 500.0);
        assert!((d2.y.atan2(d2.x).to_degrees() - d0.y.atan2(d0.x).to_degrees() - 20.0).abs() < 1e-6);
    }

    #[test]
    fn the_arm_fades_with_the_hand_and_hides_on_the_credit() {
        let mut p = hand_project();
        p.timeline.slices = vec![Slice { id: 1, start: 0.0, end: 2000.0, hide_cursor: true, ..Default::default() }];
        let mut s = Scene::new(p.clone(), &recording(), 60.0);
        let f = s.frame(1000.0, 1080.0);
        assert!(f.arm.unwrap().alpha < 0.05, "a per slice hide fades the arm");
        p.timeline.slices.clear();
        p.style.credit = true;
        let mut s = Scene::new(p, &recording(), 60.0);
        assert!(s.frame(s.duration() - 500.0, 1080.0).arm.is_none(), "the credit has no arm");
    }

    #[test]
    fn the_sleeve_width_follows_the_hand() {
        let mut s = Scene::new(hand_project(), &recording(), 60.0);
        let small = s.frame(1000.0, 600.0).arm.unwrap().width;
        let large = s.frame(1000.0, 1200.0).arm.unwrap().width;
        assert!(large > small * 1.9, "the width scales with the hand");
    }

    #[test]
    fn the_daisy_mark_turns_every_few_seconds() {
        let rec = recording();
        let mut s = Scene::new(Project::default(), &rec, 60.0);
        assert_eq!(s.frame(0.0, 1080.0).watermark_spin, 0.0, "starts upright");
        assert!(s.frame(300.0, 1080.0).watermark_spin > 0.2, "turns mid-cycle");
        assert_eq!(s.frame(3000.0, 1080.0).watermark_spin, 0.0, "rests between turns");
        // The turn finishes just under a full circle, then rests.
        assert!(s.frame(1180.0, 1080.0).watermark_spin > 6.0);
        assert!(s.frame(4300.0, 1080.0).watermark_spin > 0.2, "repeats");
    }

    #[test]
    fn reframe_keeps_the_action_in_frame() {
        use crate::engine::geom::v2;
        use crate::project::model::{Aspect, ZoomMode};
        let rec = recording();
        let mut p = Project::default();
        p.style.aspect = Aspect::Ratio { w: 9, h: 16 };
        // A zoom on something at the far right: in a vertical frame it would
        // otherwise sit against the edge.
        let target = v2(0.95, 0.5);
        p.timeline.zooms.push(Zoom { start: 0.0, end: 8000.0, scale: 2.0, mode: ZoomMode::Manual { target }, ..Default::default() });
        let focus_px = |s: &Scene| s.sizes.crop.at(target);
        let width = Scene::new(p.clone(), &rec, 60.0).frame(3000.0, 900.0).size.x;
        let centre = width / 2.0;

        p.style.auto_reframe = false;
        let mut off = Scene::new(p.clone(), &rec, 60.0);
        let f_off = off.frame(3000.0, 900.0);
        let off_x = off.screen_to_output(focus_px(&off), f_off.window.rect).x;
        p.style.auto_reframe = true;
        let mut on = Scene::new(p, &rec, 60.0);
        let f_on = on.frame(3000.0, 900.0);
        let on_x = on.screen_to_output(focus_px(&on), f_on.window.rect).x;

        assert!(f_on.window.rect.x <= 0.5 && f_on.window.rect.max().x >= width - 0.5, "still covers the frame");
        assert!(
            (on_x - centre).abs() < (off_x - centre).abs() - 1.0,
            "reframe brings the action nearer the middle: on {on_x:.0}, off {off_x:.0}, centre {centre:.0}"
        );
    }

    #[test]
    fn the_credit_card_ends_the_video() {
        let rec = recording();
        let mut p = Project::default();
        let mut s = Scene::new(p.clone(), &rec, 60.0);
        assert_eq!(s.duration(), 13_400.0, "the recording plus the credit");
        let mid = s.frame(6000.0, 720.0);
        assert!(!mid.credit && mid.daisy.is_none());
        // While the daisy spins it is centred; once the words appear it has
        // slid left to make room.
        let spinning = s.frame(10_600.0, 720.0);
        assert!(spinning.credit && spinning.daisy.is_some());
        let spin_x = spinning.daisy.as_ref().unwrap().center.x;
        let words = s.frame(12_900.0, 720.0);
        let word_x = words.daisy.as_ref().unwrap().center.x;
        assert!(word_x < spin_x - 10.0, "the daisy slides left: {spin_x} → {word_x}");
        assert!(words.texts.iter().any(|t| t.text.contains("Made in DaisyCap")));
        // Turning it off takes it out of the timeline.
        p.style.credit = false;
        let mut s = Scene::new(p, &rec, 60.0);
        assert_eq!(s.duration(), 10_000.0);
        assert!(!s.frame(10_500.0, 720.0).credit);
    }

    #[test]
    fn a_brush_mask_paints_dabs() {
        use crate::project::model::{Mask, MaskKind};
        let rec = recording();
        let mut p = Project::default();
        p.timeline.masks.push(Mask {
            start: 0.0,
            end: 5000.0,
            kind: MaskKind::Brush,
            dabs: vec![[0.3, 0.3, 0.05], [0.4, 0.35, 0.05]],
            ..Default::default()
        });
        let mut s = Scene::new(p, &rec, 60.0);
        let f = s.frame(1000.0, 1080.0);
        assert_eq!(f.masks.len(), 2, "one soft circle per dab");
        assert!(f.masks.iter().all(|m| m.kind == MaskKind::Blur), "painted as blur");
        for m in &f.masks {
            assert!((m.rect.w - m.rect.h).abs() < 0.5, "square bounds for a circle: {:?}", m.rect);
            assert!((m.radius - m.rect.w / 2.0).abs() < 0.5, "radius makes it a circle");
        }
        assert!(f.masks[1].rect.x > f.masks[0].rect.x, "dabs stay where they were painted");
        assert!(s.frame(6000.0, 1080.0).masks.is_empty(), "and end with the range");
    }

    #[test]
    fn the_credit_word_wipes_in_beside_the_daisy() {
        let rec = recording();
        let mut p = Project::default();
        p.style.credit = true;
        let mut s = Scene::new(p, &rec, 60.0);
        // The credit starts at 10 s here: the daisy spins first, with no word.
        assert!(s.frame(11_000.0, 720.0).texts.is_empty(), "no word while the daisy is still");
        let mid = s.frame(12_000.0, 720.0);
        let word = &mid.texts[0];
        assert!(word.left, "the word starts where the daisy ends");
        let r = word.reveal.expect("it is being revealed");
        assert!(r > 0.05 && r < 0.95, "mid-wipe: {r}");
        assert!(word.blur > 0.0, "and still coming into focus");
        let done = s.frame(12_900.0, 720.0);
        assert!((done.texts[0].reveal.unwrap() - 1.0).abs() < 1e-9, "fully there");
        assert!(done.texts[0].blur.abs() < 1e-9, "and sharp");
        // Measured with the real font, the daisy and the word sit centred together.
        let size_px: f64 = 720.0 * 0.16;
        let font = (size_px * 0.36).round().max(4.0) as f32;
        let text_w = crate::fonts::line_width("Made in DaisyCap", font) as f64;
        let pair_centre = done.daisy.as_ref().unwrap().center.x + (size_px * 0.35 + text_w) / 2.0;
        assert!((pair_centre - done.size.x / 2.0).abs() < 0.5, "centred: {pair_centre:.1} vs {:.1}", done.size.x / 2.0);
        // The daisy leaves a streak while it slides, and none while it spins.
        assert!(s.frame(11_700.0, 720.0).daisy.unwrap().velocity.x < -1.0, "a streak while sliding");
        assert_eq!(s.frame(10_600.0, 720.0).daisy.unwrap().velocity.x, 0.0, "none while spinning");
    }

    #[test]
    fn a_short_card_still_shows_the_word() {
        let rec = recording();
        let mut p = Project::default();
        p.style.credit = true;
        // The shortest card the slider allows: the word must still land on it.
        p.style.credit_ms = 2000.0;
        let mut s = Scene::new(p, &rec, 60.0);
        assert_eq!(s.duration(), 12_000.0, "the recording plus a 2 s card");
        let held = s.frame(11_600.0, 720.0);
        let word = &held.texts[0];
        assert!((word.reveal.unwrap() - 1.0).abs() < 1e-9, "fully wiped in");
        assert!(word.alpha > 0.99, "and not yet fading: {}", word.alpha);
    }

    #[test]
    fn cards_bookend_the_frame() {
        let rec = recording();
        let mut p = Project::default();
        p.style.intro = Some(Card { title: "Hello".into(), subtitle: "world".into(), ms: 2000.0 });
        p.style.outro = Some(Card { title: "Bye".into(), subtitle: String::new(), ms: 1500.0 });
        p.style.credit = false;
        let mut s = Scene::new(p, &rec, 60.0);
        assert_eq!(s.duration(), 13_500.0, "cards add to the length");

        let intro = s.frame(1000.0, 1080.0);
        assert!(intro.card, "intro is a card");
        assert!(intro.cursor.is_none() && intro.camera.is_none() && intro.masks.is_empty());
        assert!(intro.texts.iter().any(|t| t.text == "Hello"));
        assert!(intro.texts.iter().any(|t| t.text == "world"));

        let mid = s.frame(4000.0, 1080.0);
        assert!(!mid.card, "the recording still plays between the cards");
        assert!(mid.cursor.is_some());

        let outro = s.frame(12_500.0, 1080.0);
        assert!(outro.card, "outro is a card");
        assert!(outro.texts.iter().any(|t| t.text == "Bye"));
        assert!(outro.texts.iter().all(|t| t.text != "world"), "outro has its own text");
    }
}
