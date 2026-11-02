//! Output geometry: how big the video is and where the window sits.
//!
//! Sizes are computed in "design" pixels at the recording's scale; export
//! scales the whole design uniformly to the requested height.

use super::device::{self, Margins};
use super::geom::{Rect, Vec2, v2};
use crate::project::model::Style;

/// Largest output (VideoToolbox HEVC's limit); bigger designs are scaled
/// down. Native Retina and 5K/6K recordings fit unscaled.
pub const MAX_OUTPUT: Vec2 = v2(8192.0, 4320.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sizes {
    /// Recording (video) size.
    pub recording: Vec2,
    /// Crop in video pixels.
    pub crop: Rect,
    /// Inner border between window edge and content.
    pub inset: f64,
    /// Cropped screen plus inset on both sides.
    pub window: Vec2,
    /// Background padding on each side.
    pub pad: f64,
    /// Final design output size.
    pub output: Vec2,
    /// Extra centring space added by a forced aspect ratio.
    pub aspect_pad: Vec2,
    /// Output minus padding.
    pub content: Vec2,
    /// Window scale at zoom 1.
    pub initial_zoom: f64,
    /// Space taken by a device frame around the window.
    pub frame: Margins,
}

impl Sizes {
    pub fn new(style: &Style, recording: Vec2) -> Self {
        let c = style.crop;
        let crop = Rect::new(c.x * recording.x, c.y * recording.y, c.w * recording.x, c.h * recording.y);
        let inset = style.inset.padding.max(0.0);
        let window = crop.size() + Vec2::splat(2.0 * inset);
        let frame = device::margins(style.device, window);
        let framed = window + frame.size();
        let pad = (window.avg() * style.padding.max(0.0) / 100.0).ceil();
        let natural = framed + Vec2::splat(2.0 * pad);
        let output = match style.aspect.ratio() {
            Some(r) if (natural.aspect() - r).abs() > 1e-3 => natural.expand_to_aspect(r),
            _ => natural,
        };
        let content = output - Vec2::splat(2.0 * pad);
        let aspect_pad = (content - framed) / 2.0 + v2(frame.l, frame.t);
        let fill = content.div(framed);
        let initial_zoom = if output.y > output.x || pad <= 0.0 { fill.max_elem() } else { 1.0 };
        Sizes { recording, crop, inset, window, pad, output, aspect_pad, content, initial_zoom, frame }
    }

    /// Unzoomed window rect within the content area.
    pub fn default_body(&self) -> Rect {
        Rect::from_min_size(self.aspect_pad, self.window)
    }

    /// How far to shift a window so `focus` (in output px) sits at the centre
    /// of the frame, without letting the background show: clamped so the
    /// window still covers the frame along each axis. `rect` and `frame` are
    /// in the same (scaled) pixels.
    pub fn reframe_pan(&self, rect: Rect, frame: Vec2, focus: Vec2) -> Vec2 {
        let out = Rect::from_min_size(Vec2::ZERO, frame);
        let centre = out.center();
        let axis = |lo: f64, hi: f64, out_lo: f64, out_hi: f64, want: f64| {
            if hi - lo >= out_hi - out_lo {
                // Wider than the frame: keep it covering.
                want.clamp(out_hi - hi, out_lo - lo)
            } else {
                want.clamp(out_lo - lo, out_hi - hi)
            }
        };
        v2(
            axis(rect.x, rect.x + rect.w, out.x, out.x + out.w, centre.x - focus.x),
            axis(rect.y, rect.y + rect.h, out.y, out.y + out.h, centre.y - focus.y),
        )
    }

    /// Even pixel dimensions for an encoder at a given height (0 = native).
    pub fn render_size(&self, height: u32) -> (u32, u32) {
        let height = if height == 0 { self.native_height() } else { height };
        let s = height as f64 / self.output.y;
        let even = |v: f64| ((v / 2.0).round() as u32 * 2).max(2);
        (even(self.output.x * s), even(height as f64))
    }

    /// Native render height: design pixels, one per recorded pixel (capped
    /// to MAX_OUTPUT).
    pub fn native_height(&self) -> u32 {
        let cap = (MAX_OUTPUT.x / self.output.x).min(MAX_OUTPUT.y / self.output.y).min(1.0);
        (self.output.y * cap).round() as u32
    }
}

#[cfg(test)]
mod tests {    use super::*;
    use crate::project::model::Aspect;

    #[test]
    fn reframe_pan_keeps_the_window_covering() {
        let style = Style { aspect: Aspect::Ratio { w: 9, h: 16 }, ..Default::default() };
        let s = Sizes::new(&style, v2(1920.0, 1080.0));
        let frame = v2(506.0, 900.0);
        // A window wider and taller than the frame can pan, but must keep covering it.
        let rect = Rect::new(-200.0, -100.0, 900.0, 1100.0);
        let pan = s.reframe_pan(rect, frame, v2(500.0, 500.0));
        assert!(pan.x < 0.0, "moves toward a right-hand focus: {pan:?}");
        assert!(rect.x + pan.x <= 0.5 && rect.max().x + pan.x >= frame.x - 0.5, "still covers x: {pan:?}");
        assert!(rect.y + pan.y <= 0.5 && rect.max().y + pan.y >= frame.y - 0.5, "still covers y: {pan:?}");
        // A window smaller than the frame just moves toward the focus.
        let small = Rect::new(100.0, 100.0, 200.0, 200.0);
        let pan = s.reframe_pan(small, frame, Vec2::ZERO);
        assert!(pan.x > 0.0 && pan.y > 0.0, "moves toward the focus: {pan:?}");
    }

    #[test]
    fn padding_and_auto_aspect() {
        let style = Style::default();
        let s = Sizes::new(&style, v2(2000.0, 1000.0));
        assert_eq!(s.pad, 150.0);
        assert_eq!(s.output, v2(2300.0, 1300.0));
        assert_eq!(s.initial_zoom, 1.0);
        assert_eq!(s.default_body(), Rect::new(0.0, 0.0, 2000.0, 1000.0));
    }

    #[test]
    fn forced_aspect_centres() {
        let mut style = Style::default();
        style.aspect = Aspect::Ratio { w: 1, h: 1 };
        let s = Sizes::new(&style, v2(2000.0, 1000.0));
        assert_eq!(s.output, v2(2300.0, 2300.0));
        assert_eq!(s.aspect_pad, v2(0.0, 500.0));
    }

    #[test]
    fn device_frame_adds_room() {
        let mut style = Style::default();
        style.device = crate::project::model::DeviceFrame::Laptop;
        let s = Sizes::new(&style, v2(2000.0, 1000.0));
        assert!(s.output.x > 2300.0 && s.output.y > 1300.0);
        let body = s.default_body();
        assert!(body.x >= s.frame.l - 1e-9 && body.y >= s.frame.t - 1e-9);
        assert!((body.w - 2000.0).abs() < 1e-9);
    }

    #[test]
    fn no_padding_fills() {
        let mut style = Style::default();
        style.padding = 0.0;
        style.aspect = Aspect::Ratio { w: 16, h: 9 };
        let s = Sizes::new(&style, v2(1920.0, 1200.0));
        assert!(s.initial_zoom > 1.0);
        assert_eq!(s.render_size(1080), (1920, 1080));
    }
}
