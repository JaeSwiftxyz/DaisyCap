# Hand Cursor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A fourth cursor skin, "Hand": an illustrated hand whose suit sleeve always runs off the frame edge.

**Architecture:** The hand is a sprite like the other skins. The engine adds an `ArmLayer` to the frame (wrist, exit point, width, alpha) and the renderer draws the sleeve with the existing callout shaft mode, under the hand. The exit point comes from the frame's own size, so the sleeve always leaves an edge.

**Tech Stack:** Rust 2024, egui, wgpu. All art is drawn in code.

**Spec:** `docs/superpowers/specs/2026-11-10-hand-cursor-design.md`

## Global Constraints

- No third-party images; the hand and the sleeve are drawn in code.
- `CursorSkin::System` stays the default; old projects open unchanged.
- The sleeve's far point is computed from the frame, never hardcoded.
- Sleeve width and wrist offset come from the hand height, so they scale with the size setting and the zoom.
- Draw order: the sleeve goes after the window composite, before the click effects and the cursor.
- No version bump and no release in this plan. Commits are local; nothing is pushed.
- Checks: `. scripts/env.sh && cargo test`, then `git diff --check`. For pixels, the tests open a synthetic package as `tests/background.rs` does.

## Review Focus

- The cursor near a corner or an edge: the sleeve must still exit the frame, never end in a stub, and never fold back into the hand. Pinned by Task 3's `the_sleeve_always_leaves_the_frame`.
- A square or vertical output, and a 4K one: same, since the exit uses the frame's size. Pinned there too, with a forced 9:16 shape.
- A zoom in progress, when the hand is drawn scaled: the sleeve width must follow so the arm stays attached. Pinned by Task 3's `the_sleeve_width_follows_the_hand`.
- A hidden cursor, a press fade and a cut dip: the arm fades with the hand, and is absent on the cards and the credit. Pinned by Task 3's `the_arm_fades_with_the_hand_and_hides_on_the_credit` and Task 4's credit check.
- Every pointer state (arrow, ibeam, resize): all get the same hand, so no recorded cursor kind shows an armless arrow. Pinned by Task 2's mapping and Task 4's render check.

## File Structure

- `src/render/art.rs` — the hand sprite and `HAND_UNITS`.
- `src/project/model.rs` — `CursorSkin::Hand`.
- `src/app/inspector.rs` — the skin picker entry.
- `src/render/mod.rs` — the skin mapping, and the sleeve draw.
- `src/engine/scene.rs` — `ArmLayer`, `Frame.arm`, its computation.
- `tests/hand_cursor.rs` — the rendered pixel check.
- `PROGRESS.md` — one dated line.

---

### Task 1: The hand sprite

**Files:**
- Modify: `src/render/art.rs` (add `HAND_UNITS`, `hand_cursor`, and a tests module if there is none)

**Interfaces:**
- Produces: `pub const HAND_UNITS: f64 = 20.0` (the hand's height in sprite units, so the drawn hand matches a standard arrow) and `pub fn hand_cursor() -> (Image, Vec2, f64)`, the sprite, the normalised fingertip hotspot, and the height scale `(units.y + 2.0 * pad) / HAND_UNITS`, the same shape `vector_cursor` returns.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn hand_cursor_is_a_solid_hand_with_the_tip_ahead() {
    let (img, hot, scale) = hand_cursor();
    assert!(img.width > 0 && img.height > 0);
    assert!(hot.x > 0.0 && hot.x < 1.0 && hot.y > 0.0 && hot.y < 1.0, "{hot:?}");
    let alpha = |x: usize, y: usize| img.rgba[(y * img.width as usize + x) * 4 + 3];
    let opaque = img.rgba.chunks_exact(4).filter(|p| p[3] > 200).count();
    assert!(opaque > (img.width * img.height / 8) as usize, "the hand fills a good part of the sprite");
    assert!(alpha(0, 0) < 10, "the top-left corner stays clear");
    let tip = ((hot.x + 0.05) * img.width as f64) as usize;
    assert!(alpha(tip.min(img.width as usize - 1), 2) > 200, "the fingertip is solid");
    assert!((scale - (22.0 + 8.0) / HAND_UNITS).abs() < 1e-6, "scale matches the sprite height");
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `. scripts/env.sh && cargo test --lib art::tests::hand_cursor`
Expected: FAIL, `cannot find function hand_cursor`.

- [ ] **Step 3: Implement the sprite**

Draw into a sprite of about 13 by 22 units with the usual `pad = 4.0`, returning an `Image` sized `(units.x + 2 * pad, units.y + 2 * pad)` like `vector_cursor`. Per pixel, compose these parts back to front with anti-aliased coverage the way `paint` does:

- a soft black shadow at 0.28, offset down and right,
- a thin dark outline round the whole hand, so it reads on light backgrounds,
- the index finger, a capsule from the tip near (2.0, 0.0) to (4.6, 8.2), radius 1.5,
- the palm, a rounded rect at (3.2, 6.4) of 7.4 by 4.6, radius 2.4,
- three curled fingers off the palm's right, capsules near (9.6, 7.2) to (10.8, 10.8) spaced 1.3 apart, radius 1.25,
- the thumb, a capsule from (2.6, 7.6) to (0.8, 10.4), radius 1.3,
- a wrist stub, a rounded rect at (3.6, 10.0) of 5.6 by 2.6, radius 1.2, so the sprite ends at the wrist. The cuff belongs to the arm.

Fill with warm skin `#E7B48F` and shade the lower half toward `#D49A73` for form. Hotspot at the fingertip: `v2((2.0 + pad) / (units.x + 2.0 * pad), pad / (units.y + 2.0 * pad))`.

- [ ] **Step 4: Run the test to verify it passes**

Run: `. scripts/env.sh && cargo test --lib art::tests::`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/render/art.rs
git commit -m "feat(cursor): the hand sprite for the new skin"
```

---

### Task 2: The skin, its mapping and its picker entry

**Files:**
- Modify: `src/project/model.rs` (`CursorSkin`), `src/render/mod.rs` (`skin_key` and both skin matches), `src/app/inspector.rs` (the Cursor tab), `PROGRESS.md`

**Interfaces:**
- Consumes: `art::hand_cursor()` (Task 1).
- Produces: `CursorSkin::Hand`, serde name `"hand"`.

- [ ] **Step 1: Write the failing test**

In `src/project/model.rs`, in the tests module (add one if there is none):

```rust
#[test]
fn the_hand_skin_round_trips_and_stays_opt_in() {
    assert_eq!(CursorSkin::default(), CursorSkin::System);
    assert_eq!(serde_json::to_string(&CursorSkin::Hand).unwrap(), "\"hand\"");
    assert_eq!(serde_json::from_str::<CursorSkin>("\"hand\"").unwrap(), CursorSkin::Hand);
    let old: CursorStyle = serde_json::from_str("{}").unwrap();
    assert_eq!(old.skin, CursorSkin::System, "an old project opens to System");
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `. scripts/env.sh && cargo test --lib model::tests::the_hand_skin`
Expected: FAIL, no variant `Hand`.

- [ ] **Step 3: Add the variant, the mapping and the entry**

- `CursorSkin`: add `Hand` with a doc line, keeping `#[default] System`.
- `src/render/mod.rs`: `skin_key` gains `CursorSkin::Hand => 3`; both skin matches gain `CursorSkin::Hand => art::hand_cursor()`, the recorded-cursor loop and the empty fallback. The match ignores the recorded cursor's name, so every pointer state gets the hand.
- `src/app/inspector.rs`: the Cursor section's `segmented` gains `(CursorSkin::Hand, "Hand")` first, and its caption mentions that the sleeve reaches the frame edge.
- `PROGRESS.md`: one dated line: the hand skin, sleeve to the frame edge, drawn in code.

- [ ] **Step 4: Run the tests**

Run: `. scripts/env.sh && cargo test --lib`
Expected: PASS (the skin match is exhaustive again).

- [ ] **Step 5: Commit**

```bash
git add src/project/model.rs src/render/mod.rs src/app/inspector.rs PROGRESS.md
git commit -m "feat(cursor): a hand skin in the picker, drawn by the renderer"
```

---

### Task 3: The arm layer

**Files:**
- Modify: `src/engine/scene.rs` (an `ArmLayer` beside `CursorLayer`, `Frame.arm`, and the computation inside `Scene::frame`)

**Interfaces:**
- Consumes: `CursorSkin::Hand` (Task 2), `art::HAND_UNITS` (Task 1).
- Produces: `pub struct ArmLayer { pub from: Vec2, pub to: Vec2, pub width: f64, pub alpha: f64 }` and `Frame.arm: Option<ArmLayer>`.

Module-level constants in `src/engine/scene.rs`, with the file's other consts: `ARM_DEG: f64 = 48.0`, `ARM_WRIST: Vec2 = v2(0.18, 0.72)` (from the fingertip, in hand heights, x right and y down), `ARM_WIDTH: f64 = 0.50`.

- [ ] **Step 1: Write the failing tests**

In `src/engine/scene.rs` tests. `recording()` is already there; add a variation whose single mouse move sits where a test wants it, copying `recording()`'s meta. Add `use crate::project::model::Aspect;` where the tests already import model types.

```rust
fn rec_with_cursor(x: f64, y: f64) -> Recording {
    let mut r = recording();
    r.mouse = vec![MouseEvent { t: 0.0, x, y, kind: MouseKind::Move, button: 0, cursor: 0 }];
    r
}

fn hand_project() -> Project {
    let mut p = Project::default();
    p.style.cursor.skin = CursorSkin::Hand;
    p
}

#[test]
fn the_sleeve_always_leaves_the_frame() {
    for (x, y) in [(60.0, 60.0), (1860.0, 60.0), (60.0, 1020.0), (1860.0, 1020.0), (960.0, 540.0)] {
        for aspect in [Aspect::Auto, Aspect::Ratio { w: 9, h: 16 }] {
            let mut p = hand_project();
            p.style.aspect = aspect;
            let mut s = Scene::new(p, &rec_with_cursor(x, y), 60.0);
            let f = s.frame(500.0, 1080.0);
            let arm = f.arm.expect("the hand skin draws an arm");
            assert!(arm.to.x < -arm.width || arm.to.x > f.size.x + arm.width || arm.to.y < -arm.width || arm.to.y > f.size.y + arm.width, "{x},{y} -> {arm:?}");
        }
    }
    // No arm on the other skins.
    let mut s = Scene::new(Project::default(), &recording(), 60.0);
    assert!(s.frame(500.0, 1080.0).arm.is_none());
}

#[test]
fn the_arm_ignores_tilt_but_follows_base_rotation() {
    let rec = recording();
    let dir = |s: &mut Scene, t: f64| {
        let a = s.frame(t, 1080.0).arm.unwrap();
        (a.to - a.from).normalized()
    };
    let mut s = Scene::new(hand_project(), &rec, 60.0);
    let (d0, d1) = (dir(&mut s, 500.0), dir(&mut s, 2500.0));
    assert!((d0 - d1).length() < 1e-6, "a tilt never turns the arm");
    let mut p = hand_project();
    p.style.cursor.base_rotation = 20.0;
    let d2 = dir(&mut Scene::new(p, &rec, 60.0), 500.0);
    assert!((d2.y.atan2(d2.x).to_degrees() - d0.y.atan2(d0.x).to_degrees() - 20.0).abs() < 1e-6);
}

#[test]
fn the_arm_fades_with_the_hand_and_hides_on_the_credit() {
    let mut p = hand_project();
    p.timeline.slices = vec![Slice { id: 1, start: 0.0, end: 2000.0, hide_cursor: true, ..Default::default() }];
    let mut s = Scene::new(p.clone(), &recording(), 60.0);
    let f = s.frame(1000.0, 1080.0);
    assert!(f.arm.unwrap().alpha < 0.05, "a per slice hide fades the arm");
    p.timeline.slices.clear();
    p.style.credit = true;
    let mut s = Scene::new(p, &recording(), 60.0);
    assert!(s.frame(s.duration() - 500.0, 1080.0).arm.is_none(), "the credit has no arm");
}

#[test]
fn the_sleeve_width_follows_the_hand() {
    let mut s = Scene::new(hand_project(), &recording(), 60.0);
    let small = s.frame(1000.0, 600.0).arm.unwrap().width;
    let large = s.frame(1000.0, 1200.0).arm.unwrap().width;
    assert!(large > small * 1.9, "the width scales with the hand");
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `. scripts/env.sh && cargo test --lib the_sleeve_always the_arm_ignores the_arm_fades the_sleeve_width`
Expected: FAIL, no field `arm`.

- [ ] **Step 3: Implement the layer**

Add the struct and the `Frame` field, both with a doc line. In `Scene::frame`'s cursor block, when `style.cursor.skin == CursorSkin::Hand` and a cursor exists, compute the hand height once (`let hand = cursor::BASE_HEIGHT * style.cursor.size * k * zoom_scale;`, the same expression the cursor layer's height uses), build the arm, and add `use crate::render::art;` to the file if it is not there:

```rust
let rot = cv[2].to_radians();
let off = ARM_WRIST * (hand * art::HAND_UNITS / 20.0);
let wrist = pos + Vec2::new(off.x * rot.cos() - off.y * rot.sin(), off.x * rot.sin() + off.y * rot.cos());
let a = (ARM_DEG + style.cursor.base_rotation).to_radians();
let dir = Vec2::new(a.cos(), a.sin());
frame.arm = Some(ArmLayer { from: wrist, to: wrist + dir * (size.x + size.y), width: ARM_WIDTH * hand * art::HAND_UNITS / 20.0, alpha: fv[0].clamp(0.0, 1.0) });
```

Wherever the frame drops the cursor (the card and credit blocks), clear `frame.arm` too, and in the cut dip block multiply `arm.alpha` by the same dip factor the cursor takes.

- [ ] **Step 4: Run the tests**

Run: `. scripts/env.sh && cargo test --lib scene::tests`
Expected: PASS, including the existing ones.

- [ ] **Step 5: Commit**

```bash
git add src/engine/scene.rs
git commit -m "feat(cursor): an arm layer whose sleeve reaches the frame edge"
```

---

### Task 4: Draw the sleeve, and prove it in pixels

**Files:**
- Modify: `src/render/mod.rs` (the `render` draw list)
- Create: `tests/hand_cursor.rs`

**Interfaces:**
- Consumes: `Frame.arm` (Task 3).
- Produces: nothing later tasks use.

- [ ] **Step 1: Write the failing test**

`tests/hand_cursor.rs`, copying the small `editor` and `draw` helpers from `tests/background.rs` (synthetic package, `Editor::open`, `preview.draw`, `read_pixels`). It compares the Hand skin against System: everything that differs is the hand and its arm, so a difference on the image's outer border means the sleeve is cut by the frame edge.

```rust
//! The Hand skin's sleeve is drawn over the window and leaves the frame edge.

#[test]
fn the_sleeve_is_drawn_and_cut_by_the_frame_edge() {
    let (_dir, mut ed) = editor("hand");
    let draw_with = |ed: &mut Editor, skin: CursorSkin| {
        let mut d = ed.project.clone();
        d.style.cursor.skin = skin;
        d.timeline.zooms.clear();
        ed.commit(d);
        ed.seek(0.0);
        draw(ed, (640, 400))
    };
    let hand = draw_with(&mut ed, CursorSkin::Hand);
    let system = draw_with(&mut ed, CursorSkin::System);
    let differs = |x: usize, y: usize| {
        hand.rgba[(y * hand.width as usize + x) * 4..][..3] != system.rgba[(y * system.width as usize + x) * 4..][..3]
    };
    let changed = (0..hand.height as usize).flat_map(|y| (0..hand.width as usize).map(move |x| (x, y))).filter(|&(x, y)| differs(x, y)).count();
    assert!(changed > 400, "the hand and its arm are drawn");
    let (w, h) = (hand.width as usize, hand.height as usize);
    let on_border = (0..w)
        .flat_map(|x| [(x, 0), (x, h - 1)])
        .chain((0..h).flat_map(|y| [(0, y), (w - 1, y)]))
        .filter(|&(x, y)| differs(x, y))
        .count();
    assert!(on_border > 0, "the sleeve reaches an edge, so the border shows it");
}

#[test]
fn the_credit_has_no_arm() {
    let (_dir, mut ed) = editor("hand-credit");
    let mut d = ed.project.clone();
    d.style.cursor.skin = CursorSkin::Hand;
    ed.commit(d);
    let t = ed.duration() - 100.0;
    ed.preview.draw(t, (640, 400));
    assert!(ed.preview.last_frame.as_ref().unwrap().arm.is_none(), "the credit has no arm");
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `. scripts/env.sh && cargo test --test hand_cursor`
Expected: FAIL, the sleeve is not drawn so the sample point matches the System skin.

- [ ] **Step 3: Draw it**

In `src/render/mod.rs`'s `render`, right after the window composite block and before "Click circles under the cursor", draw the arm with `MODE_CALLOUT`'s shaft branch, skipping under alpha 0.002:

- a bounding rect over the wrist and the arm's `to`, inset by the thickness;
- `d.out[3] = 1.0` selects the shaft, `d.p3 = [from.x, from.y, to.x, to.y]`, `d.p0 = [thickness, thickness, 0.0, alpha]`, `d.p4 = colour`, `d.p0[3] = arm.alpha`;
- first the cuff: from the wrist to the wrist plus 0.9 of the arm's width along the direction, thickness 1.25 times the arm's width, colour `#F2F1EE`;
- then the sleeve over it: from the wrist plus 0.4 of the cuff's length along the direction to the arm's `to`, thickness the arm's width, colour `#202127`;
- then a thin highlight `#3A3B44` along the sleeve, offset up and left by a third of the width, half the sleeve's thickness.

The shaft's taper at `to` falls off frame, and the cuff's taper hides inside the sleeve.

- [ ] **Step 4: Run the test and the whole suite**

Run: `. scripts/env.sh && cargo test --test hand_cursor`, then `. scripts/env.sh && cargo test`
Expected: PASS.

- [ ] **Step 5: Render stills for the record**

```bash
. scripts/env.sh
[ -x dist/daisycap ] || scripts/build.sh
D=$(mktemp -d); dist/daisycap synth "$D/h.daisycap" --duration 3
mkdir -p .artifacts/hand-cursor
dist/daisycap frame "$D/h.daisycap" -t 1.5 --height 720 --style '{"style":{"cursor":{"skin":"hand"}}}' -o .artifacts/hand-cursor/wide.png
dist/daisycap frame "$D/h.daisycap" -t 1.5 --height 900 --style '{"style":{"aspect":{"type":"ratio","w":9,"h":16},"cursor":{"skin":"hand"}}}' -o .artifacts/hand-cursor/vertical.png
```

Open the two PNGs and check the sleeve runs to an edge in both.

- [ ] **Step 6: Commit**

```bash
git add src/render/mod.rs tests/hand_cursor.rs
git commit -m "feat(cursor): draw the hand's sleeve to the frame edge"
```

---

## Self-Review

- Spec coverage: art (Task 1), skin, picker and docs (Task 2), the arm layer and its rules (Task 3), draw order plus pixels (Task 4).
- Names are used the same way throughout: `hand_cursor`, `HAND_UNITS`, `ArmLayer { from, to, width, alpha }`, `Frame.arm`, `ARM_DEG`, `ARM_WRIST`, `ARM_WIDTH`.
- Every Review Focus line points at a test in the task that owns the code.
- No version bump and no push, as the spec says.
