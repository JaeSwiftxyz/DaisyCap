//! The end credit reaches the exported video: once the daisy has slid aside,
//! "Made in DaisyCap" is on screen beside it. Read back from the MP4 itself.

use std::sync::atomic::AtomicBool;

use daisycap::export::{self, Job, Source};
use daisycap::media::FrameSource;
use daisycap::project::Package;
use daisycap::project::model::{ExportSettings, Quality};

#[test]
fn the_exported_video_shows_the_word() {
    let dir = std::env::temp_dir().join(format!("daisycap-credit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Two seconds of recording, then the default 3.4 s credit.
    let path = dir.join("Credit.daisycap");
    daisycap::synth::write_package(&path, 2.0, 640, 400).unwrap();
    // With sound, as a typical recording has (system audio on by default).
    daisycap::synth::add_tone(&path, 880.0).unwrap();
    let (project, recording) = Package::new(&path).load().unwrap();
    let output = dir.join("out.mp4");
    let settings = ExportSettings { height: 400, fps: 30, quality: Quality::Web, ..Default::default() };
    let job = Job { project, recording, source: Source::Package(path.clone()), settings, output: output.clone() };
    export::run(&job, &mut |_| {}, &AtomicBool::new(false)).unwrap();

    // 2.0 s is where the card starts; by 4.6 s the word has wiped in and sits still.
    let mut src = FrameSource::open(&output).unwrap();
    let audio = daisycap::media::read_audio(&output, 0).unwrap();
    println!("video {:.2} s, audio {:.2} s", src.duration_ms() / 1000.0, audio.len() as f64 / 48_000.0);
    let frame = src.frame_at(4600.0).expect("a frame on the end credit");
    let bright = frame.with_bgra(|v| {
        let (w, h) = (v.width as usize, v.height as usize);
        let mut n = 0;
        for y in (h * 2 / 5)..(h * 3 / 5) {
            for x in (w * 2 / 5)..(w * 17 / 20) {
                let i = y * v.stride + x * 4;
                let (b, g, r) = (v.bgra[i], v.bgra[i + 1], v.bgra[i + 2]);
                if r > 200 && g > 200 && b > 200 {
                    n += 1;
                }
            }
        }
        n
    });
    assert!(bright > 200, "the word is on screen in the exported video: {bright} bright pixels in its place");

    let _ = std::fs::remove_dir_all(&dir);
}
