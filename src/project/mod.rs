//! `.daisycap` packages: a folder holding the editable project and the
//! untouched recording.
//!
//! ```text
//! Name.daisycap/
//!   project.json        style, timeline, export settings
//!   recording/
//!     meta.json         what was recorded
//!     screen.mov        screen video (no cursor) + audio tracks
//!     camera.mov        camera video, if recorded
//!     mouse.json        mouse events
//!     keys.json         key presses
//!     cursors.json      cursor images (PNG files in cursors/)
//!     transcript.json   captions, once transcribed
//! ```

pub mod model;
pub mod presets;
pub mod recording;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::engine::zoom;
use model::{DeviceFrame, Project, Slice, Style};
use recording::{AudioTrack, Recording, RecordingMeta, SourceKind, VideoTrack};

pub const EXTENSION: &str = "daisycap";

#[derive(Clone, Debug)]
pub struct Package {
    pub root: PathBuf,
}

impl Package {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Package { root: root.into() }
    }
    pub fn project_file(&self) -> PathBuf {
        self.root.join("project.json")
    }
    pub fn recording_dir(&self) -> PathBuf {
        self.root.join("recording")
    }
    pub fn rec(&self, name: &str) -> PathBuf {
        self.recording_dir().join(name)
    }
    pub fn name(&self) -> String {
        self.root.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    }

    pub fn load_recording(&self) -> Result<Recording> {
        let meta: RecordingMeta = read_json(&self.rec("meta.json"))?;
        Ok(Recording {
            meta,
            mouse: read_json_or_default(&self.rec("mouse.json"))?,
            keys: read_json_or_default(&self.rec("keys.json"))?,
            cursors: read_json_or_default(&self.rec("cursors.json"))?,
            transcript: read_json_or_default(&self.rec("transcript.json"))?,
        })
    }

    pub fn load_project(&self) -> Result<Project> {
        read_json(&self.project_file())
    }

    pub fn load(&self) -> Result<(Project, Recording)> {
        let rec = self.load_recording()?;
        let project = match self.load_project() {
            Ok(p) => p,
            Err(_) => new_project(&self.name(), &rec),
        };
        Ok((project, rec))
    }

    /// Copy a file into the package's `assets/` folder; returns the path
    /// relative to the package, so projects stay self-contained.
    pub fn import_asset(&self, src: &Path, stem: &str) -> Result<String> {
        let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("bin").to_ascii_lowercase();
        let dir = self.root.join("assets");
        std::fs::create_dir_all(&dir)?;
        let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis());
        let name = format!("{stem}-{ms}.{ext}");
        std::fs::copy(src, dir.join(&name)).with_context(|| format!("copying {}", src.display()))?;
        Ok(format!("assets/{name}"))
    }

    pub fn save_project(&self, p: &Project) -> Result<()> {
        write_json(&self.project_file(), p)
    }
}

/// A fresh project for a recording: one slice covering everything,
/// automatic zooms on clicks, and a style suited to the source.
pub fn new_project(name: &str, rec: &Recording) -> Project {
    let mut p = Project { name: name.into(), ..Default::default() };
    p.id = format!("{:016x}", rec.meta.created_ms as u64 ^ 0x9e37_79b9_7f4a_7c15);
    p.created_ms = rec.meta.created_ms;
    p.updated_ms = rec.meta.created_ms;
    if rec.meta.kind == SourceKind::Display {
        // Full-screen recordings sit on the background too (a little less
        // padding than windows), so the background settings show from the
        // start; set Padding to 0 % for an edge-to-edge video.
        p.style.padding = Style::REVEAL_PADDING;
    }
    if rec.meta.kind == SourceKind::Device {
        // iPhone/iPad screens get a device frame; there's no cursor to follow.
        let (w, h) = (rec.meta.screen.width, rec.meta.screen.height);
        p.style.device = if h > w { DeviceFrame::Phone } else { DeviceFrame::Tablet };
        p.style.padding = 8.0;
    }
    let id = p.timeline.new_id();
    p.timeline.slices.push(Slice { id, start: 0.0, end: rec.meta.duration_ms, ..Default::default() });
    let events = crate::engine::events::Events::new(rec.mouse.clone());
    let zooms = zoom::auto_zooms(&events.downs, rec.meta.duration_ms, p.style.default_zoom, &mut p.timeline);
    p.timeline.zooms = zooms;
    p
}

/// Import a video file as a new editable package: the file becomes the screen
/// track and a fresh project is laid over it.
pub fn import_video(src: &Path, projects_dir: &Path) -> Result<Package> {
    let (w, h, fps, duration_ms) = crate::media::video_info(src).with_context(|| format!("reading {}", src.display()))?;
    anyhow::ensure!(w > 0 && h > 0 && duration_ms > 1.0, "that file has no video track");
    let stem = src.file_stem().map(|s| s.to_string_lossy().into_owned()).filter(|s| !s.is_empty()).unwrap_or_else(|| "Imported".into());
    let dir = unique_dir(projects_dir, &stem);
    let pkg = Package::new(&dir);
    fs::create_dir_all(pkg.recording_dir())?;
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("mov").to_ascii_lowercase();
    let file = format!("screen.{ext}");
    // Hard link when we can (no copy of a large file), else copy.
    if fs::hard_link(src, pkg.rec(&file)).is_err() {
        fs::copy(src, pkg.rec(&file)).with_context(|| format!("copying {}", src.display()))?;
    }
    // Keep the file's own sound: the system-audio track points back into the
    // video, so the editor can play, mix and transcribe it.
    let has_audio = crate::media::audio_track_count(src).unwrap_or(0) > 0;
    let created_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis()) as i64;
    let meta = RecordingMeta {
        version: 1,
        kind: SourceKind::Display,
        duration_ms,
        screen: VideoTrack { file: file.clone(), width: w, height: h, fps, offset_ms: 0.0 },
        system_audio: has_audio.then(|| AudioTrack { file: file.clone(), track: 0 }),
        scale: if h > 1200 { 1.0 } else { 2.0 },
        created_ms,
        ..Default::default()
    };
    write_json(&pkg.rec("meta.json"), &meta)?;
    let rec = Recording { meta, ..Default::default() };
    let project = new_project(&pkg.name(), &rec);
    pkg.save_project(&project)?;
    Ok(pkg)
}

/// A `<stem>.daisycap` path that doesn't exist yet, `-2`, `-3`… as needed.
fn unique_dir(dir: &Path, stem: &str) -> PathBuf {
    let mut p = dir.join(format!("{stem}.{EXTENSION}"));
    let mut n = 2;
    while p.exists() {
        p = dir.join(format!("{stem}-{n}.{EXTENSION}"));
        n += 1;
    }
    p
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {    let s = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&s).with_context(|| format!("parsing {}", path.display()))
}

fn read_json_or_default<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    if path.exists() { read_json(path) } else { Ok(T::default()) }
}

/// Write JSON atomically (temp file + rename) so a crash never leaves a
/// half-written project.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_vec_pretty(value)?;
    fs::write(&tmp, data).with_context(|| format!("writing {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("saving {}", path.display()))?;
    Ok(())
}

/// Default folder for new projects.
pub fn projects_dir() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join("Movies").join("DaisyCap")
}

/// A new, unused package path like `~/Movies/DaisyCap/Recording 2026-10-11 at 11.30.12.daisycap`.
pub fn new_package_path(dir: &Path, stamp: &str) -> PathBuf {
    let base = format!("Recording {stamp}");
    let mut path = dir.join(format!("{base}.{EXTENSION}"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{base} {n}.{EXTENSION}"));
        n += 1;
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use recording::{MouseEvent, MouseKind};

    #[test]
    fn new_project_has_slice_and_zooms() {
        let rec = Recording {
            meta: RecordingMeta { duration_ms: 10_000.0, kind: SourceKind::Window, ..Default::default() },
            mouse: vec![MouseEvent { t: 2000.0, x: 1.0, y: 1.0, kind: MouseKind::Down, button: 0, cursor: 0 }],
            ..Default::default()
        };
        let p = new_project("x", &rec);
        assert_eq!(p.timeline.slices.len(), 1);
        assert_eq!(p.timeline.zooms.len(), 1);
        assert_eq!(p.style.padding, 10.0);
    }

    #[test]
    fn every_kind_of_recording_starts_with_the_background_in_view() {
        for kind in [SourceKind::Display, SourceKind::Window, SourceKind::Area, SourceKind::Device] {
            let rec = Recording { meta: RecordingMeta { duration_ms: 1000.0, kind, ..Default::default() }, ..Default::default() };
            let p = new_project("x", &rec);
            assert!(p.style.padding > 0.0, "{kind:?}: padding {}", p.style.padding);
            assert!(p.style.shadow.intensity > 0.0, "{kind:?}");
        }
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join(format!("daisycap-test-{}", std::process::id()));
        fs::create_dir_all(dir.join("recording")).unwrap();
        let pkg = Package::new(&dir);
        write_json(&pkg.rec("meta.json"), &RecordingMeta { duration_ms: 1000.0, ..Default::default() }).unwrap();
        let (p, rec) = pkg.load().unwrap();
        pkg.save_project(&p).unwrap();
        assert_eq!(pkg.load_project().unwrap(), p);
        assert_eq!(rec.meta.duration_ms, 1000.0);
        fs::remove_dir_all(dir).unwrap();
    }
}
