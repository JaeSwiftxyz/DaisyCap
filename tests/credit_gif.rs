//! The end credit reaches a GIF export too: the GIF's delta frames are
//! composited back into whole frames, and the one showing 4.6 s has the word.

use std::sync::atomic::AtomicBool;

use daisycap::export::{self, Job, Source};
use daisycap::project::Package;
use daisycap::project::model::{ExportFormat, ExportSettings, Quality};

#[test]
fn the_gif_shows_the_word() {
    let dir = std::env::temp_dir().join(format!("daisycap-gif-credit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let path = dir.join("Credit.daisycap");
    daisycap::synth::write_package(&path, 2.0, 640, 400).unwrap();
    let (project, recording) = Package::new(&path).load().unwrap();
    let output = dir.join("out.gif");
    let settings = ExportSettings { format: ExportFormat::Gif, height: 480, fps: 15, quality: Quality::Studio, gif_loop: true, ..Default::default() };
    let job = Job { project, recording, source: Source::Package(path.clone()), settings, output: output.clone() };
    export::run(&job, &mut |_| {}, &AtomicBool::new(false)).unwrap();

    let mut opts = gif::DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::RGBA);
    let mut dec = opts.read_info(std::fs::File::open(&output).unwrap()).unwrap();
    let (w, h) = (dec.width() as usize, dec.height() as usize);
    let mut canvas = vec![0u8; w * h * 4];
    let (mut clock, mut at_target, mut frames) = (0u64, None, 0);
    while let Some(frame) = dec.read_next_frame().unwrap() {
        frames += 1;
        let (fl, ft, fw) = (frame.left as usize, frame.top as usize, frame.width as usize);
        for y in 0..frame.height as usize {
            for x in 0..fw {
                let si = (y * fw + x) * 4;
                if frame.buffer[si + 3] == 0 {
                    continue;
                }
                let di = ((ft + y) * w + fl + x) * 4;
                canvas[di..di + 4].copy_from_slice(&frame.buffer[si..si + 4]);
            }
        }
        // Centiseconds: the frame showing at 4.6 s is the last one that starts by then.
        let start = clock;
        clock += frame.delay as u64;
        if start <= 460 {
            at_target = Some(canvas.clone());
        }
    }
    println!("{frames} frames, {:.2} s", clock as f64 / 100.0);
    let shown = at_target.expect("a frame at 4.6 s");

    let mut bright = 0;
    for y in (h * 2 / 5)..(h * 3 / 5) {
        for x in (w * 2 / 5)..(w * 17 / 20) {
            let i = (y * w + x) * 4;
            if shown[i] > 200 && shown[i + 1] > 200 && shown[i + 2] > 200 {
                bright += 1;
            }
        }
    }
    let evidence = concat!(env!("CARGO_MANIFEST_DIR"), "/.artifacts/gif-credit.png");
    let _ = image::save_buffer(evidence, &shown, w as u32, h as u32, image::ExtendedColorType::Rgba8);
    assert!(bright > 200, "the word is in the GIF at 4.6 s: {bright} bright pixels in its place");

    let _ = std::fs::remove_dir_all(&dir);
}
