//! The Hand skin's sleeve is drawn over the window and leaves the frame edge.
//! Everything that differs between the Hand frame and a System one is the
//! hand and its arm, so a difference on the image border means the frame cut
//! the sleeve.

use std::path::PathBuf;

use daisycap::app::editor::Editor;
use daisycap::project::model::CursorSkin;
use daisycap::render::{Gpu, Image};

/// A synthetic recording opened in the editor, with its auto zooms removed.
fn editor(name: &str) -> (PathBuf, Editor) {
    let dir = std::env::temp_dir().join(format!("daisycap-hand-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("Hand.daisycap");
    daisycap::synth::write_package(&path, 1.0, 1280, 800).unwrap();
    let mut ed = Editor::open(&path, Gpu::headless().unwrap()).unwrap();
    let mut d = ed.project.clone();
    d.timeline.zooms.clear();
    ed.commit(d);
    (dir, ed)
}

fn draw(ed: &mut Editor, size: (u32, u32)) -> Image {
    ed.preview.draw(0.0, size);
    let (width, height, rgba) = ed.preview.read_pixels().expect("preview pixels");
    Image { width, height, rgba }
}

#[test]
fn the_sleeve_is_drawn_and_cut_by_the_frame_edge() {
    let (dir, mut ed) = editor("live");
    let draw_with = |ed: &mut Editor, skin: CursorSkin| {
        let mut d = ed.project.clone();
        d.style.cursor.skin = skin;
        ed.commit(d);
        ed.seek(0.0);
        draw(ed, (640, 400))
    };
    let hand = draw_with(&mut ed, CursorSkin::Hand);
    let system = draw_with(&mut ed, CursorSkin::System);

    let differs = |x: usize, y: usize| {
        hand.rgba[(y * hand.width as usize + x) * 4..][..3] != system.rgba[(y * system.width as usize + x) * 4..][..3]
    };
    let (w, h) = (hand.width as usize, hand.height as usize);
    let changed = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).filter(|&(x, y)| differs(x, y)).count();
    assert!(changed > 400, "the hand and its arm are drawn, {changed} pixels changed");
    let on_border = (0..w)
        .flat_map(|x| [(x, 0), (x, h - 1)])
        .chain((0..h).flat_map(|y| [(0, y), (w - 1, y)]))
        .filter(|&(x, y)| differs(x, y))
        .count();
    assert!(on_border > 0, "the sleeve reaches an edge, so the border shows it");

    drop(ed);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_credit_has_no_arm() {
    let (dir, mut ed) = editor("credit");
    let mut d = ed.project.clone();
    d.style.cursor.skin = CursorSkin::Hand;
    ed.commit(d);
    let t = ed.duration() - 100.0;
    ed.preview.draw(t, (640, 400));
    assert!(ed.preview.last_frame.as_ref().unwrap().arm.is_none(), "the credit has no arm");

    drop(ed);
    let _ = std::fs::remove_dir_all(&dir);
}
