//! The "Share link" window: the finished export and its poster go to
//! jaeswift.xyz, and the link goes on the clipboard. The first time, the app
//! claims a handle (the name in the link) and keeps the token it's given.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::JoinHandle;

use anyhow::{Result, bail};
use egui::{Align2, Button, ProgressBar, RichText, TextEdit, vec2};

use super::editor::Editor;
use super::home::Settings;
use super::{macos, widgets};
use crate::share::{self, Details, Shared};

#[derive(Default)]
pub struct ShareState {
    pub open: bool,
    /// The title the video's page shows.
    pub title: String,
    /// A handle being typed, before it is claimed.
    pub draft: String,
    /// The last thing that happened, for the window to say.
    pub note: Option<String>,
    /// The video that is up, for Copy, Open and Remove.
    pub last: Option<Shared>,
    run: Option<Run>,
}

struct Run {
    sent: Arc<AtomicU64>,
    total: u64,
    cancel: Arc<AtomicBool>,
    thread: JoinHandle<Result<Shared>>,
}

impl ShareState {
    /// An upload is in flight.
    pub fn busy(&self) -> bool {
        self.run.is_some()
    }
}

impl Editor {
    /// Start uploading the finished export. The poster is the frame under the playhead.
    fn start_share(&mut self, handle: &str, token: &str) -> Result<()> {
        let (path, width, height) = match &self.export.result {
            Some(Ok((path, stats))) => (path.clone(), stats.width, stats.height),
            _ => bail!("export the video first, then share it"),
        };
        let poster = self.preview.poster_jpeg(self.playhead);
        let details = Details { title: self.share.title.trim().to_owned(), width, height, duration_ms: self.duration() };
        let total = std::fs::metadata(&path)?.len();
        let sent = Arc::new(AtomicU64::new(0));
        let cancel = Arc::new(AtomicBool::new(false));
        let (s2, c2) = (sent.clone(), cancel.clone());
        let (handle, token) = (handle.to_owned(), token.to_owned());
        let thread = std::thread::spawn(move || {
            let shared = share::upload_video(&handle, &token, &details, &path, &s2, &c2)?;
            if let Some(jpeg) = poster {
                // Without its poster a link still works; it just has no preview image.
                let _ = share::upload_poster(&handle, &token, &shared.slug, &jpeg);
            }
            Ok(shared)
        });
        self.share.note = Some("Uploading…".into());
        self.share.run = Some(Run { sent, total, cancel, thread });
        Ok(())
    }

    /// Collect a finished upload: copy its link, and keep it for Open and Remove.
    pub fn poll_share(&mut self) {
        if !self.share.run.as_ref().is_some_and(|r| r.thread.is_finished()) {
            return;
        }
        let Some(run) = self.share.run.take() else { return };
        match run.thread.join() {
            Ok(Ok(shared)) => {
                macos::copy_text(&shared.url);
                self.share.note = Some("Link copied. Paste it anywhere.".into());
                self.share.last = Some(shared);
            }
            Ok(Err(e)) => self.share.note = Some(format!("{e:#}")),
            Err(_) => self.share.note = Some("Sharing stopped unexpectedly.".into()),
        }
    }

    /// The "Share link" window, shown over the editor while it is open.
    pub fn share_window(&mut self, ctx: &egui::Context, settings: &mut Settings) {
        if !self.share.open {
            return;
        }
        if self.share.title.is_empty() {
            self.share.title = self.project.name.clone();
        }
        let mut open = true;
        egui::Window::new("Share link")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .fixed_size(vec2(420.0, 0.0))
            .open(&mut open)
            .show(ctx, |ui| {
                let mut claim = None;
                match settings.share_handle.clone() {
                    Some(handle) => {
                        ui.horizontal(|ui| {
                            ui.label("Your links start with");
                            ui.label(RichText::new(format!("jaeswift.xyz/daisycap/{handle}/")).strong());
                        });
                    }
                    None => {
                        widgets::caption(ui, "Pick a handle: the name in your links. This Mac claims it once.");
                        ui.horizontal(|ui| {
                            ui.add(TextEdit::singleline(&mut self.share.draft).desired_width(200.0).hint_text("your-name"));
                            let typed = self.share.draft.trim().to_owned();
                            let ok = share::valid_handle(&typed);
                            if ui.add_enabled(ok, Button::new("Claim")).clicked() {
                                claim = Some(typed);
                            }
                        });
                        if !self.share.draft.trim().is_empty() && !share::valid_handle(self.share.draft.trim()) {
                            widgets::caption(ui, "3 to 30 letters, digits and single hyphens. Not api, admin or daisycap.");
                        }
                    }
                }
                if let Some(handle) = claim {
                    match share::claim(&handle, None) {
                        Ok(Some(token)) => {
                            settings.share_handle = Some(handle.clone());
                            settings.share_token = Some(token);
                            settings.save();
                            self.share.note = Some(format!("Claimed {handle}. It belongs to this Mac."));
                        }
                        Ok(None) => self.share.note = Some("That handle is already yours.".into()),
                        Err(e) => self.share.note = Some(format!("{e:#}")),
                    }
                }

                ui.add_space(8.0);
                widgets::caption(ui, "Links last one hour. Then the video is deleted from the server.");
                ui.label("Title");
                ui.add(TextEdit::singleline(&mut self.share.title).desired_width(f32::INFINITY));
                let exported = matches!(self.export.result, Some(Ok(_)));
                let ready = exported && settings.share_handle.is_some() && !self.share.busy();
                if !exported {
                    widgets::caption(ui, "Export the video first, then share it.");
                }
                ui.horizontal(|ui| {
                    if ui.add_enabled(ready, Button::new("Upload and copy link")).clicked() {
                        let handle = settings.share_handle.clone().unwrap_or_default();
                        let token = settings.share_token.clone().unwrap_or_default();
                        if let Err(e) = self.start_share(&handle, &token) {
                            self.share.note = Some(format!("{e:#}"));
                        }
                    }
                });

                if let Some(run) = &self.share.run {
                    let sent = run.sent.load(Ordering::Relaxed) as f32;
                    let frac = if run.total > 0 { (sent / run.total as f32).clamp(0.0, 1.0) } else { 0.0 };
                    ui.add(ProgressBar::new(frac).show_percentage());
                    if ui.button("Cancel").clicked() {
                        run.cancel.store(true, Ordering::Relaxed);
                    }
                }

                if let Some(last) = self.share.last.clone() {
                    ui.separator();
                    ui.label(RichText::new(&last.url).monospace());
                    ui.horizontal(|ui| {
                        if ui.button("Copy link").clicked() {
                            macos::copy_text(&last.url);
                        }
                        if ui.button("Open").clicked() {
                            macos::open_url(&last.url);
                        }
                        if ui.button("Remove link").clicked() {
                            match settings.share_token.clone() {
                                Some(token) => match share::remove(&last.handle, &token, &last.slug) {
                                    Ok(()) => {
                                        self.share.last = None;
                                        self.share.note = Some("Link removed.".into());
                                    }
                                    Err(e) => self.share.note = Some(format!("{e:#}")),
                                },
                                None => self.share.note = Some("No token on this Mac for that link.".into()),
                            }
                        }
                    });
                }
                if let Some(note) = &self.share.note {
                    widgets::caption(ui, note);
                }
            });
        if !open {
            self.share.open = false;
        }
    }
}
