//! The launch splash: DaisyCap's daisy turns clockwise and settles upright,
//! then fades and the window's content takes its place. A click or key press
//! skips it, and the scripted GUI checks start without it.

use std::f64::consts::TAU;
use std::time::Instant;

use egui::emath::Rot2;
use egui::{Color32, Mesh, Pos2, Rect, TextureHandle, Ui, vec2};
use objc2_app_kit::NSWorkspace;

/// Seconds the splash stays up.
pub const TOTAL_SECS: f64 = 1.8;
/// When the daisy has settled and the fade begins.
const SETTLE_SECS: f64 = 1.4;
/// Whole turns before settling. The daisy has eight petals, which repeat every
/// 45 degrees, so two turns end exactly where the logo sits.
const TURNS: f64 = 2.0;
/// Dark in both themes, like the app icon, so the white petals read on light Macs.
const TOP: Color32 = Color32::from_rgb(0x3c, 0x3f, 0x47);
const BOTTOM: Color32 = Color32::from_rgb(0x0d, 0x0e, 0x11);

/// The daisy's angle `t` seconds in, in radians, positive clockwise on screen.
/// It eases out: quickest at the start, and it comes to rest upright.
pub fn angle_at(t: f64, reduce_motion: bool) -> f32 {
    if reduce_motion {
        return 0.0;
    }
    let p = (t / SETTLE_SECS).clamp(0.0, 1.0);
    (TURNS * TAU * (1.0 - (1.0 - p).powi(3))) as f32
}

/// The daisy's opacity: full until it settles, then fading out by `TOTAL_SECS`.
pub fn alpha_at(t: f64) -> f32 {
    ((TOTAL_SECS - t) / (TOTAL_SECS - SETTLE_SECS)).clamp(0.0, 1.0) as f32
}

/// The daisy mark as a texture.
pub fn mark_texture(ctx: &egui::Context) -> TextureHandle {
    let px = crate::render::icon::render_mark(256);
    ctx.load_texture("splash-daisy", egui::ColorImage::from_rgba_unmultiplied([256, 256], &px), egui::TextureOptions::LINEAR)
}

/// Paint the splash over all of `ui`, `t` seconds in.
pub fn paint(ui: &mut Ui, mark: &TextureHandle, t: f64, reduce_motion: bool) {
    let full = ui.max_rect();
    ui.painter().add(background(full));
    let side = (full.width().min(full.height()) * 0.3).clamp(96.0, 200.0);
    let rect = Rect::from_center_size(full.center(), vec2(side, side));
    let c = rect.center();
    let turn = Rot2::from_angle(angle_at(t, reduce_motion));
    let tint = Color32::WHITE.gamma_multiply(alpha_at(t));
    let corners = [rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()];
    let uvs = [Pos2::new(0.0, 0.0), Pos2::new(1.0, 0.0), Pos2::new(1.0, 1.0), Pos2::new(0.0, 1.0)];
    let mut daisy = Mesh::with_texture(mark.id());
    for (corner, uv) in corners.into_iter().zip(uvs) {
        daisy.vertices.push(egui::epaint::Vertex { pos: c + turn * (corner - c), uv, color: tint });
    }
    daisy.add_triangle(0, 1, 2);
    daisy.add_triangle(0, 2, 3);
    ui.painter().add(daisy);
}

fn background(rect: Rect) -> Mesh {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), TOP);
    mesh.colored_vertex(rect.right_top(), TOP);
    mesh.colored_vertex(rect.right_bottom(), BOTTOM);
    mesh.colored_vertex(rect.left_bottom(), BOTTOM);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    mesh
}

/// One window's splash. It starts on the window's first frame and ends when
/// its time is up, or when the visitor clicks or presses a key.
pub struct Splash {
    /// When the first frame was drawn, for the fallback start.
    first_frame: Option<Instant>,
    /// When the window was first on screen. The clock starts here.
    started: Option<Instant>,
    on_screen: bool,
    reduce_motion: bool,
    mark: Option<TextureHandle>,
    done: bool,
}

impl Splash {
    pub fn new() -> Splash {
        let scripted = crate::util::env_os("DAISYCAP_GUI_SELFTEST").is_some() || crate::util::env_os("DAISYCAP_GUI_PLAYTEST").is_some();
        Splash { first_frame: None, started: None, on_screen: false, reduce_motion: false, mark: None, done: scripted }
    }

    /// Whether the window is on screen yet, as seen before the next frame.
    pub fn observe(&mut self, on_screen: bool) {
        self.on_screen = on_screen;
    }

    /// Whether the splash still has frames to show.
    pub fn running(&self) -> bool {
        !self.done
    }

    /// Paint the splash over the window while it runs. Returns true while it
    /// covers the window, so the caller draws nothing else.
    pub fn show(&mut self, ui: &mut Ui) -> bool {
        if self.done {
            return false;
        }
        let now = Instant::now();
        let first = *self.first_frame.get_or_insert(now);
        // The clock starts once the window is on screen. If it never reports
        // that, start after a second rather than cover the window for good.
        if self.started.is_none() && (self.on_screen || now.duration_since(first).as_secs_f64() > 1.0) {
            self.started = Some(now);
            self.reduce_motion = NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion();
        }
        let t = self.started.map_or(0.0, |s| now.duration_since(s).as_secs_f64());
        let skip = ui.input(|i| i.pointer.any_pressed() || i.events.iter().any(|e| matches!(e, egui::Event::Key { pressed: true, .. })));
        if skip || t >= TOTAL_SECS {
            self.done = true;
            return false;
        }
        let mark = self.mark.get_or_insert_with(|| mark_texture(ui.ctx()));
        paint(ui, mark, t, self.reduce_motion);
        ui.ctx().request_repaint();
        true
    }
}

impl Default for Splash {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_4;

    #[test]
    fn turns_clockwise_on_screen() {
        // Twelve o'clock moves towards three o'clock when the daisy turns clockwise.
        let turned = Rot2::from_angle(angle_at(0.3, false)) * vec2(0.0, -1.0);
        assert!(turned.x > 0.0, "{turned:?}");
    }

    #[test]
    fn settles_upright_and_stays_there() {
        // Eight petals repeat every 45 degrees, so a whole number of them is upright.
        let rest = angle_at(SETTLE_SECS, false) as f64;
        let eighths = rest / FRAC_PI_4;
        assert!((eighths - eighths.round()).abs() < 1e-4, "{rest}");
        assert_eq!(angle_at(TOTAL_SECS, false), angle_at(SETTLE_SECS, false));
    }

    #[test]
    fn eases_out() {
        let early = angle_at(0.1, false) - angle_at(0.0, false);
        let late = angle_at(SETTLE_SECS, false) - angle_at(SETTLE_SECS - 0.1, false);
        assert!(early > 10.0 * late, "early {early}, late {late}");
    }

    #[test]
    fn fades_out_after_settling() {
        assert_eq!(alpha_at(0.5), 1.0);
        assert!(alpha_at(1.6) > 0.0 && alpha_at(1.6) < 1.0);
        assert!(alpha_at(TOTAL_SECS) < 1e-6);
    }

    #[test]
    fn reduced_motion_holds_still() {
        assert_eq!(angle_at(0.5, true), 0.0);
        assert_eq!(angle_at(TOTAL_SECS, true), 0.0);
    }
}
