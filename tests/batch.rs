//! "All three shapes" exports the recording as 16:9, square and vertical in
//! one go, so a single take covers every feed.

use std::time::{Duration, Instant};

use daisycap::app::editor::Editor;
use daisycap::project::model::{ExportFormat, Quality};
use daisycap::render::Gpu;

#[test]
fn the_social_batch_writes_three_shapes() {
    let dir = std::env::temp_dir().join(format!("daisycap-batch-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("Clip.daisycap");
    daisycap::synth::write_package(&path, 1.5, 320, 200).unwrap();

    let mut ed = Editor::open(&path, Gpu::headless().unwrap()).unwrap();
    ed.project.export = daisycap::project::model::ExportSettings { format: ExportFormat::Mp4, height: 200, fps: 10, quality: Quality::Web, ..Default::default() };
    ed.start_social_batch();

    let start = Instant::now();
    while (ed.export.running.is_some() || !ed.export.batch.is_empty()) && start.elapsed() < Duration::from_secs(90) {
        std::thread::sleep(Duration::from_millis(30));
        ed.tick();
    }
    assert!(ed.export.running.is_none() && ed.export.batch.is_empty(), "the batch finished");

    let names = ["Clip-16x9.mp4", "Clip-square.mp4", "Clip-vertical.mp4"];
    let mut shapes = vec![];
    for n in names {
        let p = dir.join(n);
        assert!(p.exists(), "{n} was written");
        let (w, h, _, _) = daisycap::media::video_info(&p).unwrap();
        shapes.push(w as f64 / h as f64);
    }
    assert!((shapes[0] - 16.0 / 9.0).abs() < 0.02, "{shapes:?}");
    assert!((shapes[1] - 1.0).abs() < 0.02, "{shapes:?}");
    assert!((shapes[2] - 9.0 / 16.0).abs() < 0.02, "{shapes:?}");

    let _ = std::fs::remove_dir_all(&dir);
}
