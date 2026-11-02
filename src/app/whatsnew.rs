//! The "What's new" sheet: shown once after an update, and from the Help
//! menu on demand. Content is a plain list of what changed in this version.

use egui::RichText;

use super::theme;
use super::widgets::caption;

/// The app version, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// What's new in this version, newest first.
pub const NOTES: &[(&str, &[&str])] = &[(
    "0.2.4",
    &[
        "Share links: upload an export to jaeswift.xyz and copy its link, jaeswift.xyz/daisycap/<you>/<video>. Claim your handle once; it stays with this Mac.",
    ],
), (
    "0.2.3",
    &[
        "Intro and outro cards: a title card before the recording and an end card after it, drawn on your background.",
        "Callouts: arrows, boxes and numbered step pins that follow the zoom.",
        "Audio worth keeping: a waveform under the timeline, one-click silence trim, and fades so cuts don't click.",
        "Speed ramps and dip-at-cuts transitions.",
        "Captions can be saved as .srt or .vtt for YouTube and the web.",
        "Export presets for X, LinkedIn, Shorts and 4K, and one-click frame shapes.",
        "Drop any MP4 or MOV on the window to edit it, not just DaisyCap recordings.",
        "The timeline fits its tracks in a short window, and the demo project chimes instead of droning.",
        "This sheet now opens from a ? in the toolbar, Help › What's New…, and the DaisyCap menu-bar icon.",
        "Imported videos keep their sound, and captions transcribe the audio track when there is no microphone.",
        "A dark end credit: the daisy spins, slides aside and \"Made in DaisyCap\" wipes in beside it.",
        "\"All three shapes\" exports 16:9, square and vertical in one go, and the frame follows the action when you pick a shape.",
        "A brand kit (your logo and two colours), caption styles with presets, and a freehand blur brush.",
    ],
)];

/// The sheet's open state; shown over whatever else is on screen.
#[derive(Default)]
pub struct Sheet {
    pub open: bool,
    /// Set when it was opened because the version changed.
    from_update: bool,
}

impl Sheet {
    /// Open when the version differs from the last one seen — including the
    /// first launch after this feature shipped, when nothing was recorded yet.
    pub fn after_update(&mut self, last: Option<&str>) {
        if last != Some(VERSION) {
            self.open = true;
            self.from_update = last.is_some();
        }
    }

    /// Open it on demand (Help menu): the "you updated" line doesn't apply.
    pub fn open_now(&mut self) {
        self.open = true;
        self.from_update = false;
    }

    /// True while the sheet is up (the app redraws so it isn't half-drawn).
    pub fn showing(&self) -> bool {
        self.open
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        let p = theme::palette(matches!(ctx.theme(), egui::Theme::Dark));
        let mut open = true;
        let mut dismissed = false;
        egui::Window::new("What's new")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .fixed_size(egui::vec2(460.0, 0.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(RichText::new(format!("DaisyCap {VERSION}")).font(theme::semibold(15.0)));
                if self.from_update {
                    ui.label(RichText::new("Updated while you were away. Here's what's new.").color(p.secondary));
                } else {
                    ui.label(RichText::new("Everything DaisyCap can do, at a glance.").color(p.secondary));
                }
                ui.add_space(6.0);
                for (version, notes) in NOTES {
                    ui.label(RichText::new(format!("Version {version}")).font(theme::semibold(12.0)).color(p.secondary));
                    for n in *notes {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            ui.label(RichText::new("•").color(theme::accent()));
                            ui.label(*n);
                        });
                    }
                    ui.add_space(4.0);
                }
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Got it").clicked() {
                        dismissed = true;
                    }
                });
                caption(ui, "Show it again with the ? button in the toolbar, Help › What's New…, or the menu-bar icon.");
            });
        if dismissed || !open {
            self.open = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sheet_opens_after_a_version_change_only() {
        let mut s = Sheet::default();
        s.after_update(None);
        assert!(s.showing(), "an existing install with no recorded version sees it");
        assert!(!s.from_update, "but not with the update wording");
        let mut s = Sheet::default();
        s.after_update(Some(VERSION));
        assert!(!s.showing(), "same version stays quiet");
        let mut s = Sheet::default();
        s.after_update(Some("0.1"));
        assert!(s.showing(), "a different version opens it");
        assert!(s.from_update);
    }

    #[test]
    fn notes_are_not_empty() {
        assert!(!NOTES.is_empty());
        for (v, notes) in NOTES {
            assert!(!v.is_empty());
            assert!(!notes.is_empty());
        }
    }
}
