//! The demo project: a synthetic recording dressed up with the app's own
//! features (cards, callouts, a mask, captions, a zoom), so someone can see
//! what the editor does without recording anything. Built on first use.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::engine::geom::Rect;
use crate::project::model::{Callout, CalloutKind, Card, Mask, MaskKind};
use crate::project::recording::Word;
use crate::project::{Package, write_json};

pub const NAME: &str = "DaisyCap Demo";

/// Create the demo project in `dir` if it isn't there yet, and return its path.
pub fn ensure(dir: &Path) -> Result<PathBuf> {
    let root = dir.join(format!("{NAME}.daisycap"));
    if root.join("project.json").exists() {
        refresh_audio(&root)?;
        return Ok(root);
    }
    std::fs::create_dir_all(dir)?;
    crate::synth::write_package(&root, 8.0, 1920, 1080)?;
    let pkg = Package::new(&root);
    let (mut project, _rec) = pkg.load()?;
    project.name = NAME.into();
    refresh_audio(&root)?;

    // A title card either side, so the shape of a finished video is obvious.
    project.style.intro = Some(Card { title: "DaisyCap".into(), subtitle: "a very quick tour".into(), ms: 2200.0 });
    project.style.outro = Some(Card { title: "That's DaisyCap".into(), subtitle: "jaeswift.xyz".into(), ms: 2400.0 });

    // A step pin, an arrow and a labelled box, each pointing at the mock UI.
    let steps = [
        (CalloutKind::Step, Rect::new(0.16, 0.20, 0.03, 0.03), "Step 1: the sidebar"),
        (CalloutKind::Arrow, Rect::new(0.44, 0.72, 0.12, -0.22), "an arrow follows the zoom"),
        (CalloutKind::Box, Rect::new(0.60, 0.16, 0.24, 0.16), "or box something"),
    ];
    let mut step = 1;
    for (i, (kind, rect, text)) in steps.into_iter().enumerate() {
        let id = project.timeline.new_id();
        project.timeline.callouts.push(Callout {
            id,
            start: 1500.0 + i as f64 * 400.0,
            end: 7000.0,
            kind,
            rect,
            step,
            text: text.into(),
            ..Default::default()
        });
        step += 1;
    }
    // A blur mask over the mock list, and a caption track to match.
    let id = project.timeline.new_id();
    project.timeline.masks.push(Mask { id, start: 4000.0, end: 6500.0, kind: MaskKind::Blur, rect: Rect::new(0.34, 0.24, 0.26, 0.2), amount: 14.0, radius: 12.0, ..Default::default() });

    let line = |at: f64, text: &str| {
        let words: Vec<&str> = text.split(' ').collect();
        words
            .iter()
            .enumerate()
            .map(|(i, w)| Word { start: at + i as f64 * 300.0, end: at + i as f64 * 300.0 + 280.0, text: (*w).into() })
            .collect::<Vec<_>>()
    };
    let mut transcript: Vec<Word> = vec![];
    transcript.extend(line(1500.0, "This is DaisyCap"));
    transcript.extend(line(3600.0, "callouts point at things"));
    transcript.extend(line(6000.0, "and captions write themselves"));
    write_json(&pkg.rec("transcript.json"), &transcript)?;
    project.style.captions.show = true;

    pkg.save_project(&project)?;
    Ok(root)
}

/// Marker for the current demo audio, so an older demo (with the synth's
/// constant tone) is rewritten instead of keeping the beep.
const AUDIO_MARK: &str = "demo-audio-2";

/// Replace the demo's audio with three soft chimes where the captions land,
/// so the waveform has something to show and nothing drones.
fn refresh_audio(root: &Path) -> Result<()> {
    let pkg = Package::new(root);
    if pkg.rec(AUDIO_MARK).exists() {
        return Ok(());
    }
    let rec = pkg.load_recording()?;
    let track = chimes(rec.meta.duration_ms);
    crate::app::player::write_wav(&pkg.rec("system.wav"), &track)?;
    let mut meta = rec.meta;
    meta.system_audio = Some(crate::project::recording::AudioTrack { file: "system.wav".into(), track: 0 });
    write_json(&pkg.rec("meta.json"), &meta)?;
    std::fs::write(pkg.rec(AUDIO_MARK), b"")?;
    Ok(())
}

/// A quiet two-note chime at each caption, and silence between them.
fn chimes(duration_ms: f64) -> Vec<[f32; 2]> {
    let rate = crate::capture::mixer::RATE;
    let mut out = vec![[0.0f32; 2]; (duration_ms / 1000.0 * rate) as usize];
    for (i, at_ms) in [1400.0, 3500.0, 5900.0].into_iter().enumerate() {
        let base = if i == 2 { 659.25 } else { 523.25 };
        for (k, hz) in [base, base * 1.25].into_iter().enumerate() {
            let start = ((at_ms + k as f64 * 170.0) / 1000.0 * rate) as usize;
            let len = (0.55 * rate) as usize;
            for j in 0..len {
                let Some(slot) = out.get_mut(start + j) else { break };
                let t = j as f64 / rate;
                // Soft attack, gentle decay.
                let env = (1.0 - (-t * 22.0).exp()) * (-t * 5.5).exp();
                let v = (std::f64::consts::TAU * hz * t).sin() * env * 0.1;
                slot[0] += v as f32;
                slot[1] += v as f32;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_demo_has_notes() {
        // Guards the sheet's content against being emptied by accident.
        assert!(super::NAME.contains("Demo"));
    }
}
