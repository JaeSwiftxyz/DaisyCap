# Hand cursor: a sleeve that always reaches the frame edge

Date: 2026-11-10. Branch: `agent/hand-cursor-1110a`. Task clone:
`~/Desktop/daisycap-worktrees/hand-cursor-1110a`.

## What and why

A new cursor skin for fun: an illustrated hand with a suit-sleeved arm, in the
style of the reference (a light warm hand pointing, a white cuff, a near-black
sleeve, the arm entering the frame at a diagonal and running off the edge).
It sits next to the existing skins, System, Vector and Touch, in the Cursor
tab, and is used for every pointer state.

The rule the owner set: the sleeve always runs off an edge of the frame,
from wherever the hand is, whatever the output shape. The arm is part of the
fun, not a stub of a forearm.

## Goals

- A fourth skin, `Hand`, that draws the illustrated hand instead of an arrow.
- The sleeve always leaves the frame: for any hand position and any output
  aspect, the sleeve crosses an edge and the frame cuts it.
- The hand keeps today's behaviour: smoothing, tilt, press squash, idle fade,
  per slice `hide_cursor`, the cut dip, and hiding on cards and the credit.
- The sleeve stays steady: its angle does not swing with tilt, and a press
  does not squash it.
- Drawn in code, matching the repo's rule that no third-party images are
  bundled.
- Identical on preview, thumbnails, GIF and export, which all share
  `Scene::frame` and the renderer.

## Non-goals

- No per-pointer-state art. Arrow, ibeam, resize and the rest all get the same
  hand.
- No user controls for the arm angle, thickness or colours.
- No edge-aiming (nearest edge or quadrant). One fixed diagonal.
- No dedicated blur or shadow pass for the sleeve.
- No version bump in this task. Releasing is a separate ask, and nothing is
  pushed until the owner asks.

## Anti-goals

- The sleeve ending inside the frame, a bare arm stub. That breaks the rule.
- The sleeve swinging with tilt or shrinking on press, so the hand looks
  detached from its arm.
- The sleeve being clipped by the window or the masks. It sits over the window
  and under the hand.
- Any change to the System, Vector or Touch look.

## Constraints

- `CursorSkin` is a serde enum. Add a variant, keep `System` the default, so
  existing projects open unchanged.
- Output size and aspect vary (16:9, square, vertical, up to 4K). The exit
  point is computed from the frame, never hardcoded.
- The cursor's height already follows the size setting, the output scale and
  the zoom. The sleeve thickness follows the same height so the arm stays
  attached at any zoom, while its far end still reaches the frame edge.
- Draw order in the renderer (`src/render/mod.rs`, `render`): window composite,
  click circles, cursor, camera, callouts, watermark, text. The arm goes right
  after the window composite, before the cursor and the click effects.

## Approaches

1. **Hand sprite plus an arm layer (chosen).** The hand is a sprite, as the
   other skins are. The engine works out the sleeve as a line from the wrist to
   a point beyond the frame edge, and the renderer draws it. The hand keeps its
   tilt, press and motion, the arm stays put, and the sleeve reaches the edge by
   construction.
2. **One baked sprite with a long sleeve at a fixed angle.** Less code and it
   reuses the cursor path as is, but the sleeve is part of the sprite, so tilt
   swings it and a press shortens it, and the sprite has to be oversized to
   cover every position and aspect.

Aiming the arm at the nearest edge was considered and dropped: the angle flips
when the hand crosses the frame's midline, which reads as a twitch.

Approach 1 was chosen because the rule holds by construction and the hand's
motion stays independent of the arm.

## How it works

- **Model.** `CursorSkin::Hand` (serde `"hand"`), label "Hand", first in the
  segmented control in the Cursor tab (`src/app/inspector.rs`).
- **Art.** `art::hand_cursor() -> (Image, Vec2, f64)` in `src/render/art.rs`,
  beside `vector_cursor` and `touch_cursor`. The hand points up and a little to
  the left, fingertip at the hotspot, with a warm skin tone, the palm and the
  curled fingers, a thin outline and a soft baked shadow. The sprite ends at the
  wrist, since the cuff belongs to the arm layer so that the cuff always lines
  up with the sleeve. The returned scale puts the hand at today's arrow height.
- **Engine.** `Frame` gains `arm: Option<ArmLayer>`, with
  `ArmLayer { from: Vec2, to: Vec2, width: f64, alpha: f64 }` next to
  `CursorLayer`. `Scene::frame` fills it wherever it builds a cursor:
  - the wrist is the hand position plus the wrist offset in the hand's own
    frame, rotated by the cursor's rotation, scaled by the hand height;
  - the direction is the fixed diagonal (down and to the right, about 48
    degrees below horizontal) plus `base_rotation`, so it ignores the dynamic
    tilt but still turns with a hand whose base angle was set;
  - `to` is the wrist plus the direction times `size.x + size.y`, so it leaves
    the frame from any position, by more than the drawn head can cover;
  - `width` is half the hand height, `alpha` starts as the cursor's alpha;
  - `arm` is `None` exactly where `frame.cursor` is `None` (the global hide,
    cards, the credit). A per slice or idle fade keeps `arm` present with
    alpha 0, so it fades in step with the hand instead of popping.
- **Height basis.** The renderer draws the hand at the cursor height times the
  art's returned scale, which the engine cannot see. So `art` exports the hand
  height (the same value `hand_cursor()` returns), and the engine builds the
  wrist offset and the sleeve width from `cursor height * that`, so the two
  cannot drift apart.
- **Fades.** The cut dip that dims the window, cursor and callouts scales
  `arm.alpha` too, so the whole cursor dips together.
- **Renderer.** Draw the arm before the cursor and after the window composite,
  using the existing `MODE_CALLOUT` arrow branch. The branch tapers to a point
  at `to` over the head length, so the tapers are hidden rather than seen: the
  sleeve's far point sits off frame, and the cuff runs into the sleeve's start
  so the sleeve covers the cuff's point. The sleeve's rounded start sits under
  the cuff and hand. Two draws: the white cuff and the dark sleeve, plus an
  optional thin lighter line along the sleeve for a fold. Colours in code:
  sleeve `#202127`, cuff `#F2F1EE`, highlight `#3A3B44`. No shader change.
- **Skin mapping.** `src/render/mod.rs` gains `Hand => 3` in `skin_key`, a
  `CursorSkin::Hand => art::hand_cursor()` arm in the skin match, and the same
  in the no-recorded-cursor fallback.

## Tests and evidence

- Art test: `hand_cursor` is non-empty, the hotspot is inside it, the corners
  are transparent and the fingertip area is opaque.
- Scene tests: with the Hand skin the arm ends outside the frame for the four
  corners and the centre; the direction holds steady while the tilt animates
  and follows `base_rotation`; `arm` is `None` on the credit card; a per slice
  hide leaves `arm` present with alpha near 0.
- Render test: a frame with the Hand skin has sleeve-coloured pixels along the
  path back from the hand, while System has none.
- `cargo test`, plus rendered stills from `daisycap frame` on a synthetic
  recording, including a vertical export, kept in `.artifacts/`.

## Left to the builder

- The hand's shapes, tones and outline, and the wrist offset. Tune by eye.
- The sleeve width and cuff length, the optional highlight, and the exact
  diagonal angle around 48 degrees.
- Whether a press also nudges the sleeve position (the squash itself stays on
  the hand).
- Helper names and where the arm draw helper lives.

## My calls, for review

- Fixed diagonal rather than nearest-edge aiming.
- The same hand for every pointer state.
- The cuff belongs to the arm layer, not the sprite, so it stays aligned with
  the sleeve when the hand tilts.
- The arm draws over the window and masks, under the hand and the click
  effects.
- No version bump and no release inside this task.

From the builder review, settled here rather than asked:

- The arm follows `base_rotation` but not the animated tilt, so a hand that was
  set to a base angle still keeps a straight arm.
- Art exports the hand height, so the engine's wrist offset and sleeve width
  cannot drift from the drawn hand.
- The arm is built with the cursor and nulled only where the cursor is nulled;
  a fade keeps it present at alpha 0, so it fades rather than pops.
- The cut dip scales the arm's alpha with the cursor's.
- The callout branch tapers to a point at `to`. The sleeve's point sits off
  frame and the cuff's point hides under the sleeve, so no shader change.
