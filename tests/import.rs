//! A video file can be imported as a new editable project: the file becomes
//! the screen track and a fresh project is laid over it.

use daisycap::project::{Package, import_video};

#[test]
fn importing_a_video_makes_an_editable_project() {
    let dir = std::env::temp_dir().join(format!("daisycap-import-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // A recording to borrow a real .mov from.
    let src = dir.join("Source.daisycap");
    daisycap::synth::write_package(&src, 2.0, 640, 400).unwrap();
    let video = dir.join("clip.mov");
    std::fs::copy(src.join("recording/screen.mov"), &video).unwrap();

    let out = dir.join("projects");
    std::fs::create_dir_all(&out).unwrap();
    let pkg = import_video(&video, &out).unwrap();
    assert_eq!(pkg.name(), "clip");
    assert!(pkg.project_file().exists(), "a project was written");

    let (project, rec) = Package::new(&pkg.root).load().unwrap();
    assert_eq!(rec.meta.screen.width, 640);
    assert_eq!(rec.meta.screen.height, 400);
    assert!(rec.meta.duration_ms > 1500.0, "kept the video's length");
    assert_eq!(project.timeline.slices.len(), 1, "one clip over the whole video");

    // Importing the same name again doesn't overwrite.
    let again = import_video(&video, &out).unwrap();
    assert_ne!(again.root, pkg.root);
    assert_eq!(again.name(), "clip-2");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn importing_keeps_the_videos_own_sound() {
    use daisycap::project::model::{ExportSettings, Quality};
    use std::sync::atomic::AtomicBool;

    let dir = std::env::temp_dir().join(format!("daisycap-import-audio-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // A recording with a tone, exported to an MP4 that carries its sound.
    let src = dir.join("Tone.daisycap");
    daisycap::synth::write_package(&src, 2.0, 320, 200).unwrap();
    daisycap::synth::add_tone(&src, 880.0).unwrap();
    let pkg = Package::new(&src);
    let (project, recording) = pkg.load().unwrap();
    let video = dir.join("clip.mp4");
    let settings = ExportSettings { height: 200, fps: 10, quality: Quality::Web, ..Default::default() };
    let job = daisycap::export::Job {
        project,
        recording,
        source: daisycap::export::Source::Package(src.clone()),
        settings,
        output: video.clone(),
    };
    daisycap::export::run(&job, &mut |_| {}, &AtomicBool::new(false)).unwrap();

    // Importing it must keep that sound, not drop it.
    let out = dir.join("projects");
    std::fs::create_dir_all(&out).unwrap();
    let imported = import_video(&video, &out).unwrap();
    let (_, rec) = Package::new(&imported.root).load().unwrap();
    let track = rec.meta.system_audio.expect("the imported video's audio track");
    let audio = daisycap::media::read_audio(&imported.rec(&track.file), track.track).unwrap();
    assert!(audio.iter().any(|s| s[0].abs() > 0.02), "the tone survived the import");

    let _ = std::fs::remove_dir_all(&dir);
}
