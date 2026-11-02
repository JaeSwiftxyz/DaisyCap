//! The demo project is generated with the app's own features so someone can
//! see what the editor does without recording anything.

use daisycap::app::demo;
use daisycap::project::Package;

#[test]
fn the_demo_project_shows_the_features() {
    let dir = std::env::temp_dir().join(format!("daisycap-demo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let root = demo::ensure(&dir).unwrap();
    assert_eq!(root.file_name().unwrap().to_string_lossy(), "DaisyCap Demo.daisycap");

    let (p, rec) = Package::new(&root).load().unwrap();
    assert!(p.style.intro.is_some(), "shows a title card");
    assert!(p.style.outro.is_some(), "and an end card");
    assert_eq!(p.timeline.callouts.len(), 3, "shows callouts");
    assert_eq!(p.timeline.masks.len(), 1, "and a mask");
    assert!(!p.timeline.zooms.is_empty(), "and a zoom");
    assert!(!rec.transcript.is_empty(), "with captions over it");
    assert!(rec.meta.system_audio.is_some(), "and a tone to hear");

    // The audio is three short chimes, not one long drone: mostly quiet.
    let audio = daisycap::media::read_audio(&root.join("recording/system.wav"), 0).unwrap();
    assert!(!audio.is_empty());
    let loud = audio.iter().filter(|s| s[0].abs() > 0.02).count();
    let frac = loud as f64 / audio.len() as f64;
    assert!(frac < 0.35, "chimes are spaced out, not continuous: {frac:.2}");
    assert!(loud > 1_000, "but they're audible");

    // Asking again reuses the same project rather than rebuilding it.
    assert_eq!(demo::ensure(&dir).unwrap(), root);

    let _ = std::fs::remove_dir_all(&dir);
}
