//! The feedback box: what happened, an optional email, and the diagnostics
//! that go with it. Reports go to our own endpoint; if the send fails the
//! text can be copied instead, so nothing is lost.

use std::sync::{Arc, Mutex, OnceLock};

use egui::{Align, Context, Id, Layout, RichText, TextEdit};
use serde::Serialize;

use super::theme::{self, pal};
use super::widgets;

/// Where reports are sent.
pub const ENDPOINT: &str = "https://jaeswift.xyz/api/feedback.php";

/// One report, as posted.
#[derive(Clone, Serialize)]
pub struct Report {
    pub message: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub email: String,
    pub include_diagnostics: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub diagnostics: String,
    pub version: String,
    /// Where the box was opened from: the menu, or the error banner.
    pub source: String,
}

/// Facts about this Mac and this session, for the report.
pub fn diagnostics(error: Option<&str>, session: &str) -> String {
    static STATIC: OnceLock<String> = OnceLock::new();
    let head = STATIC.get_or_init(|| {
        let macos = match (out("sw_vers", &["-productVersion"]), out("sw_vers", &["-buildVersion"])) {
            (v, b) if !v.is_empty() => format!("{v} ({b})"),
            _ => String::from("unknown"),
        };
        let chip = match (out("sysctl", &["-n", "machdep.cpu.brand_string"]), out("sysctl", &["-n", "hw.model"])) {
            (c, m) if !c.is_empty() => format!("{c} ({m})"),
            (_, m) if !m.is_empty() => m,
            _ => String::from("unknown"),
        };
        format!("DaisyCap    {}\nmacOS       {macos}\nMac         {chip}", env!("CARGO_PKG_VERSION"))
    });
    let mut text = head.clone();
    if !session.is_empty() {
        text.push_str(&format!("\nSession     {session}"));
    }
    if let Some(e) = error.filter(|e| !e.trim().is_empty()) {
        text.push_str(&format!("\nError       {}", e.replace('\n', " ")));
    }
    text
}

/// A shelled-out value, trimmed (empty when the tool is missing).
fn out(program: &str, args: &[&str]) -> String {
    std::process::Command::new(program)
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

enum State {
    Editing,
    Sending,
    Sent,
    Failed(String),
}

pub struct Form {
    open: bool,
    report: Report,
    /// What the diagnostics would be, kept for the preview.
    preview: String,
    state: State,
    /// The background send's outcome.
    result: Arc<Mutex<Option<Result<(), String>>>>,
}

impl Form {
    pub fn new() -> Form {
        Form {
            open: false,
            report: Report {
                message: String::new(),
                email: String::new(),
                include_diagnostics: true,
                diagnostics: String::new(),
                version: env!("CARGO_PKG_VERSION").into(),
                source: String::new(),
            },
            preview: String::new(),
            state: State::Editing,
            result: Arc::new(Mutex::new(None)),
        }
    }

    /// Open the box, pre-filled with `message` when it came from a failure.
    pub fn open(&mut self, source: &str, message: &str, diagnostics: String) {
        self.open = true;
        self.report.source = source.into();
        self.report.message = message.into();
        self.preview = diagnostics;
        self.state = State::Editing;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Pick up a finished send.
    fn poll(&mut self) {
        if !matches!(self.state, State::Sending) {
            return;
        }
        let done = self.result.lock().ok().and_then(|mut s| s.take());
        if let Some(outcome) = done {
            self.state = match outcome {
                Ok(()) => State::Sent,
                Err(e) => State::Failed(e),
            };
        }
    }

    fn send(&mut self) {
        self.state = State::Sending;
        let mut report = self.report.clone();
        report.diagnostics = if report.include_diagnostics { self.preview.clone() } else { String::new() };
        let slot = self.result.clone();
        std::thread::spawn(move || {
            let body = serde_json::to_string(&report).unwrap_or_default();
            let outcome = match ureq::post(ENDPOINT).set("Content-Type", "application/json").send_string(&body) {
                Ok(resp) if resp.status() == 200 => Ok(()),
                Ok(resp) => Err(format!("the server said {}", resp.status())),
                Err(e) => Err(format!("{e}")),
            };
            if let Ok(mut s) = slot.lock() {
                *s = Some(outcome);
            }
        });
    }

    /// Draw the box while it is open. Returns true while it stays open.
    pub fn show(&mut self, ctx: &Context) -> bool {
        if !self.open {
            return false;
        }
        self.poll();
        let mut keep = true;
        let modal = egui::Modal::new(Id::new("daisycap-feedback")).show(ctx, |ui| {
            ui.set_max_width(430.0);
            ui.label(RichText::new("Send feedback").font(theme::semibold(theme::BODY)));
            ui.add_space(2.0);
            ui.label(RichText::new("A bug, something confusing, or an idea. It comes straight to us.").size(theme::SECONDARY).color(pal(ui).secondary));
            ui.add_space(8.0);
            let ready = !matches!(self.state, State::Sending);
            ui.add_enabled_ui(ready, |ui| {
                ui.add(TextEdit::multiline(&mut self.report.message).desired_width(f32::INFINITY).desired_rows(5).hint_text("What happened?"));
            });
            ui.add_space(6.0);
            ui.add_enabled_ui(ready, |ui| {
                ui.horizontal(|ui| {
                    ui.add(TextEdit::singleline(&mut self.report.email).hint_text("Email (optional)").desired_width(210.0));
                    ui.checkbox(&mut self.report.include_diagnostics, "Include diagnostics");
                });
            });
            if self.report.include_diagnostics {
                ui.add_space(4.0);
                ui.collapsing("What will be sent", |ui| {
                    ui.label(RichText::new(&self.preview).size(theme::SECONDARY - 1.0).monospace().color(pal(ui).secondary));
                });
            }
            ui.add_space(10.0);
            let mut go = false;
            ui.horizontal(|ui| {
                match &self.state {
                    State::Sending => {
                        ui.spinner();
                        ui.label(RichText::new("Sending…").size(theme::SECONDARY));
                    }
                    State::Sent => {
                        ui.label(RichText::new("Sent. Thank you.").size(theme::SECONDARY).color(theme::accent()));
                    }
                    State::Failed(e) => {
                        ui.label(RichText::new(format!("Couldn't send: {e}")).size(theme::SECONDARY).color(egui::Color32::from_rgb(255, 69, 58)));
                        if ui.small_button("Copy it").clicked() {
                            ctx.copy_text(copied(&self.report, &self.preview));
                        }
                    }
                    State::Editing => {}
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let can = ready && !self.report.message.trim().is_empty();
                    ui.add_enabled_ui(can, |ui| {
                        if widgets::primary(ui, "Send").clicked() {
                            go = true;
                        }
                    });
                    if ui.button("Cancel").clicked() {
                        keep = false;
                    }
                });
            });
            go
        });
        if modal.inner {
            self.send();
        }
        if modal.should_close() && !matches!(self.state, State::Sent) {
            keep = false;
        }
        if !keep {
            self.open = false;
        }
        self.open
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

/// The whole report as text, for the clipboard.
fn copied(report: &Report, diagnostics: &str) -> String {
    let mut text = report.message.clone();
    if !report.email.is_empty() {
        text.push_str(&format!("\n\nEmail: {}", report.email));
    }
    if report.include_diagnostics {
        text.push_str(&format!("\n\n{diagnostics}"));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_name_the_app_macos_and_this_mac() {
        let d = diagnostics(Some("something broke"), "editor, 8.4 s");
        assert!(d.contains(env!("CARGO_PKG_VERSION")), "{d}");
        assert!(d.contains("macOS"), "{d}");
        assert!(d.contains("Mac "), "{d}");
        assert!(d.contains("something broke"), "{d}");
        assert!(d.contains("editor, 8.4 s"), "{d}");
        // One line per fact, so it reads in an email or an issue.
        assert!(d.lines().count() >= 5, "{d}");
        assert!(!d.lines().any(|l| l.trim().is_empty()), "{d}");
    }

    #[test]
    fn a_report_survives_json_with_quotes_and_newlines() {
        let r = Report {
            message: "It said \"no\"\nthen quit".into(),
            email: "a@b.c".into(),
            include_diagnostics: true,
            diagnostics: "line\nline".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            source: "menu".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(!json.contains('\n'), "newlines must be escaped: {json}");
        let back: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(back["message"], "It said \"no\"\nthen quit");
    }

    #[test]
    fn clipboard_text_carries_the_details_only_when_ticked() {        let mut r = Report {
            message: "broken".into(),
            email: "a@b.c".into(),
            include_diagnostics: true,
            diagnostics: "diag".into(),
            version: "1".into(),
            source: "menu".into(),
        };
        assert!(copied(&r, "diag").contains("diag"));
        r.include_diagnostics = false;
        assert!(!copied(&r, "diag").contains("diag"));
        assert!(copied(&r, "diag").contains("broken"));
    }

    /// Posts one report to the live endpoint (needs the network):
    /// `cargo test --lib app::feedback -- --ignored`.
    #[test]
    #[ignore]
    fn posts_a_report_to_the_live_endpoint() {
        let report = Report {
            message: "DaisyCap: test report from cargo test".into(),
            email: String::new(),
            include_diagnostics: true,
            diagnostics: "from cargo test".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            source: "cargo test".into(),
        };
        let body = serde_json::to_string(&report).unwrap();
        let resp = ureq::post(ENDPOINT).set("Content-Type", "application/json").send_string(&body).expect("post");
        assert_eq!(resp.status(), 200);
    }
}
