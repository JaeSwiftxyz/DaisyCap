//! Share links: the poster is a real JPEG of the frame, and a video goes up,
//! plays and comes down again on jaeswift.xyz. The live part makes real
//! uploads, so it's ignored by default:
//!   cargo test --test share -- --ignored

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use daisycap::app::editor::Editor;
use daisycap::export::{self, Job, Source};
use daisycap::project::Package;
use daisycap::project::model::{ExportSettings, Quality};
use daisycap::render::Gpu;
use daisycap::share::{self, Details};

fn package(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("Share.daisycap");
    daisycap::synth::write_package(&path, 2.0, 320, 200).unwrap();
    path
}

#[test]
fn the_poster_is_a_jpeg_of_the_frame() {
    let dir = std::env::temp_dir().join(format!("daisycap-poster-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut ed = Editor::open(&package(&dir), Gpu::headless().unwrap()).unwrap();
    let jpeg = ed.preview.poster_jpeg(1000.0).expect("a poster");
    assert_eq!(&jpeg[..3], &[0xFF, 0xD8, 0xFF], "starts like a JPEG");
    assert!(jpeg.len() > 2_000, "and holds a picture, not a flat colour: {} bytes", jpeg.len());
    drop(ed);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore]
fn a_video_goes_up_plays_and_comes_down() {
    let dir = std::env::temp_dir().join(format!("daisycap-share-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = package(&dir);
    let (project, recording) = Package::new(&path).load().unwrap();
    let mp4 = dir.join("share.mp4");
    let settings = ExportSettings { height: 200, fps: 10, quality: Quality::Web, ..Default::default() };
    let job = Job { project, recording, source: Source::Package(path.clone()), settings, output: mp4.clone() };
    export::run(&job, &mut |_| {}, &AtomicBool::new(false)).unwrap();

    let mut ed = Editor::open(&path, Gpu::headless().unwrap()).unwrap();
    let jpeg = ed.preview.poster_jpeg(1000.0).unwrap();
    drop(ed);

    let handle = format!("dc-live-{}", std::process::id() % 100_000);
    let token = share::claim(&handle, None).unwrap().expect("a new handle comes with a token");
    let details = Details { title: "Live check".into(), width: 320, height: 200, duration_ms: 2000.0 };
    let sent = AtomicU64::new(0);
    let shared = share::upload_video(&handle, &token, &details, &mp4, &sent, &AtomicBool::new(false)).unwrap();
    assert_eq!(sent.load(Ordering::Relaxed), std::fs::metadata(&mp4).unwrap().len(), "every byte went up");
    share::upload_poster(&handle, &token, &shared.slug, &jpeg).unwrap();

    let page = ureq::get(&shared.url).call().unwrap().into_string().unwrap();
    assert!(page.contains("<video"), "the page has a player");
    assert!(page.contains("og:image"), "and a preview image");
    let video = ureq::get(&format!("{}/video.mp4", shared.url)).call().unwrap();
    assert_eq!(video.content_type(), "video/mp4");

    share::remove(&handle, &token, &shared.slug).unwrap();
    match ureq::get(&shared.url).call() {
        Err(ureq::Error::Status(404, _)) => {}
        other => panic!("the page should be gone after removal: {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&dir);
    println!("test handle left on the server: {handle}");
}
