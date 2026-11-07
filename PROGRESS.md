# Progress

Product name **DaisyCap** (crate and binary `daisycap`, bundle `com.daisycap.app`).
Milestones and features are tracked here.

| # | Milestone | Status |
|---|---|---|
| M0 | Feature list and engine design | ☑ |
| M1 | Engine: springs, deterministic animator, timeline map, sizes, auto zoom + targeting, cursor model, click effects, camera layouts, shortcut badges, captions | ☑ unit tested |
| M2 | GPU renderer (wgpu/Metal): background, shadow, rounded window + inset, cursor, click FX, motion blur, camera, text, masks, device frames | ☑ |
| M3 | Media: AVAssetReader decode with seeking and prefetch | ☑ |
| M4 | Recording: ScreenCaptureKit display/window/area, system audio, mic, camera, input + cursor logging, iOS devices | ☑ (capture core from rec-rs) |
| M5 | Export: MP4 (VideoToolbox H.264/HEVC, zero-copy), audio mix with cuts/speed/voice-over ducking, GIF | ☑ |
| M6 | Editor UI (egui + Inter): one editor-first window (recording setup, recents popover), preview, inspector, timeline, recording widget, area/window pickers, menu bar, shortcuts | ☑ |
| M7 | Captions (Apple Speech, on device, editable), shortcut overlay | ☑ |
| M8 | App bundle, icon, signing (`scripts/build.sh`) | ☑ |
| M9 | Verification with real recordings; `PERFORMANCE.md` | ☑ `scripts/selftest.sh --screen`, `scripts/bench.sh` |

## Features

☑ done and verified on this Mac · ◐ done with a stated gap · ☐ not done ·
✗ deliberately different.

| ID | Feature | Status | Notes |
|---|---|---|---|
| R1 | Display recording | ☑ | Native Retina size, display refresh, cursor hidden (re-rendered later). |
| R2 | Window recording | ☑ | Window bounds logged over time; cursor stays aligned when the window moves. Picked on screen ("Pick on screen…", Record with no window chosen, or the menu bar's Record Window…): windows outlined front to back, the one under the pointer highlighted with name, size, Resize, Use/Record and Adjust area; right-click for Bring to front / Ignore / Reset ignored; Esc cancels. The pickers cover every display at once (one overlay per display; each shows its own windows, one shared selection; an area moves to the display you drag on). The scripted GUI run opens it on both displays here, highlights a window, screenshots each display and cancels. |
| R3 | Area recording | ☑ | Drag-to-select picker (move/resize, Return/Esc), size presets, exact values. |
| R4 | System audio | ☑ | Own track (stems). Everything the Mac plays, whatever the source: window recordings take it from a companion audio-only display stream (a window filter alone only hears the window's own app; before the fix a window recording was silent while another app played). ScreenCaptureKit only hears apps, not command-line players such as `afplay`. The app leaves its own sounds out; `daisycap record` doesn't (ScreenCaptureKit treats every copy of DaisyCap as one app, so excluding "itself" also muted a DaisyCap window playing). Checked by the selftest's audio round trip (a helper app plays a 1234 Hz tone, DaisyCap records it from an area and from another app's window). |
| R5 | Microphone | ☑ | Own track; "Enhance voice" applies ML voice isolation, levelling and automatic gain control (±9 dB, speech-gated) at export rather than at capture, so the raw track stays untouched. Live capture checked with Microphone granted to the terminal: both capture paths (ScreenCaptureKit and AVCaptureSession) write a 48 kHz mono track in sync with the video. On this Mac it was silent (−91 dB, same with ffmpeg) because the lid is closed and macOS disconnects the built-in mic; DaisyCap now warns about that before recording and in the voice-over panel. |
| R6 | Camera | ◐ | Separate 720p file, bubble/layouts in the editor. Not exercised live: Camera permission not granted on this Mac. |
| R7 | iOS device recording | ◐ | CoreMediaIO screen-capture devices, native-size HEVC, phone/tablet frame. Everything after capture is tested with a synthetic portrait device recording (`daisycap synth --device`: phone frame chosen automatically, no cursor, MP4 export) in `scripts/selftest.sh`. Live capture untested: no iPhone/iPad was connected. |
| R8 | Input logging | ☑ | 120 Hz mouse + buttons, cursor images with hotspots, keystrokes (needs Input Monitoring). |
| R9 | Countdown, pause/resume, restart, cancel | ☑ | Floating control bar: a native borderless, non-activating panel (clicking it doesn't bring DaisyCap forward or change the recorded menu bar) shaped as a pill with a vibrancy background, hairline border and system shadow; pulsing red dot (orange when paused), monospaced timer, SF Symbol cancel / restart / pause buttons with hover highlights and tooltips (with shortcuts), red stop button; dragged by its background, remembered for the session; centred above the Dock on the recorded display. Kept out of recordings (own-window exclusion plus `sharingType = none`): checked on a real 5K recording of the display it sat on. The scripted GUI run checks it's present and draggable. |
| R10 | Hide desktop icons / Dock | ☑ | Excluded from the capture by window layer; wallpaper kept. Verified on a real recording. |
| R11 | Global shortcuts | ☑ | Start/stop, pause, cancel; re-bindable; conflicts reported. |
| R12 | Permissions | ☑ | Status, request button, exact steps below. |
| R13 | Resize the recorded window | ☑ | Window source shows the window's size with a Resize menu: the same preset groups, saved sizes, custom size; sizes that don't fit its display are disabled. Accessibility API (needs Accessibility permission). Verified by resizing DaisyCap's own window (`scripts/selftest.sh --screen`). |
| Z1–Z7 | Auto zoom, zoom model, targeting, manual target, edge snapping, spring zoom, motion blur | ☑ | Engine unit tests; manual target picked by clicking the preview. |
| C1–C9 | Cursor re-rendering, smoothing, tilt, press, idle hide, hidden slices, click circle/ripple, click sounds, loop/freeze | ☑ | System cursor images, plus generated vector and touch skins. |
| C10 | Click and keyboard sounds | ☑ | Six synthesized click flavours, and keyboard sounds recorded from two real boards, chosen in a dropdown with an audition button; the key sounds sit in the Keys tab, the click sounds with the cursor and the whoosh with the zooms; per-key pitch and gain variation, heavier space/return, modifier keys skipped, optional typing-only filter, a zoom whoosh, and sound-mood buttons (Minimal, Office, Mech keeb). Mixed into the preview and the export; unit tests cover the bank and placement under cuts and 2× speed. |
| S1 | Backgrounds | ☑ | Generated wallpapers (soft colour fields crossed by crisp "glass" ribbons, so blur visibly softens them), gradients, colour, imported PNG/JPEG (checked on import), current desktop picture, blur. Every recording, full-screen ones included, starts with padding so the background is visible; at 0 % padding a background, blur or shadow edit brings the padding back (8 %) and says so. `daisycap check-background` and `tests/background.rs` render each option and check the pixels change, on the editor preview and on a real 5K recording; an export test checks the exported frame matches the preview. |
| S2–S7 | Padding, roundness, shadow, inset, aspect ratio, crop | ☑ | Roundness and shadow sizes are px in a 1080p-tall frame (like blur), whatever the recording's resolution, so they look the same on Retina and 1080p recordings. |
| S8 | Device frames | ☑ | Generated laptop, phone, tablet frames (no artwork reused). |
| S9 | Daisy watermark | ☑ | The icon's daisy mark in the bottom-left corner of every frame, drawn by the renderer (preview, thumbnails and export), on by default. A plain toggle turns it off; a render test checks it only changes the corner. |
| K1–K5 | Camera bubble, shape, shrink while zoomed, layouts, auto-hide | ◐ | Bubble, circle/aspect shapes and camera-only layout verified with a generated camera clip on a real recording; live camera capture unverified (permission). |
| T1–T7 | Slices, trim, split/delete, speed, zoom track, layout track, undo/redo/scrub/loop | ☑ | |
| T8 | Typing speed-up | ☑ | Detects typing runs in the key log; one click to 2–4×. |
| T9 | Masks | ☑ | Blur, pixelate, highlight; drawn on the preview; follow zoom. "Data may be visible": blur/pixelate masks are re-checked in the background (SSIM + histogram on 64×64 patches every 100 ms, one shared decode pass) and ranges where the covered content moved are flagged on the timeline; one click cuts them. Verified on a real 5K recording (only the mask over a playing video flagged, three static masks not) and by the UI smoke test. |
| T10 | Voice-over | ◐ | Recorded with AVAudioRecorder, mixed with ducking (mixing verified with a test file). Live capture needs Microphone permission. |
| O1 | Shortcut overlay | ☑ | |
| O2 | Captions | ☑ | Whisper (whisper.cpp with Metal; base/small/medium, downloaded on first use) or Apple Speech, both on device, word timings, editable lines. Verified with speech synthesized by `say` attached as a mic track: Whisper base returned all 20 words exactly, 7 s of speech in 0.4 s (plus a test that repeats this whenever the model is present). Apple Speech needs Speech Recognition permission (run from DaisyCap.app) and wasn't exercised live. |
| O3 | Intro/outro cards | ☑ | A generated title card before the recording and an end card after it, from the project's own background plus a title and optional second line, with the daisy mark. Toggled and edited in Background › Intro & outro cards; fade in/out, hold up to 10 s. The lead-in lives in the time map, so clips, zooms, masks, sounds and the recorded audio all shift together and land back in the same frames whether you seek into a card and out or play straight through (unit-tested in the map and the scene). |
| O4 | Callouts | ☑ | Pointer arrows, outlined boxes and numbered step pins over the screen, for a time range, following the window and its zoom. Toolbar "+ Callout" (C) or double-click the Callouts track; drag on the preview to draw, aim an arrow by its ends, drag the shape about; kind, number/label, colour and thickness in the inspector. A dedicated point-in-triangle shader SDF draws the arrow and box. Hidden on intro/outro cards like the rest of the recording, and re-shown with it. |
| O5 | Speed ramps & transitions | ☑ | A clip's speed can ease in and out ("Ease speed" on the selected clip): the time map fans the slice into sub-links whose speed smoothsteps from 1× to the clip speed and back, so video and audio stay in sync and no frames are lost. Cuts can dip to the background ("Dip at cuts", 60–600 ms, Motion tab): where a cut jumps the recording, the window, cursor, camera and callouts fade out and in; a plain split stays seamless. |
| A1–A2 | Mix, background music | ☑ | Per-slice volumes, mutes, master volume, looped music with fade. |
| A5 | Waveform, silence trim, slice fades | ☑ | The recorded audio's waveform under the clip track (per 5 ms window, following cuts and speed, sqrt-scaled so quiet speech shows). "Cut silences" in the Audio tab removes pauses past ~0.5 s and gives every cut a short fade; per-slice Fade in/out sliders, applied by the export and the live preview mix. Splits and cuts fade their edges automatically so they don't click. || A4 | Editor playback audio | ☑ | Plays through CoreAudio (cpal) from the edit's mix, in step with the playhead (re-seeked past 60 ms of drift; ≤ 45 ms measured). The master volume applies live in the output callback; mutes, clip volumes, music and voice-overs re-mix in the background (40–180 ms for 74 s) and swap in while playing; playing before the mix is ready picks up in step. The output restarts itself if the device changes or stops (another app reconfiguring it). Replaces AVPlayer on a temporary WAV, which stayed silent for a whole playback if the mix wasn't ready yet and ignored edits until paused. The Audio tab says when a track is silent or the mic was off. Checked by `GIMBAL_GUI_PLAYTEST` (output level, ×0.25 volume = −12.0 dB, mute = silence, seek in sync) while another recording hears the speakers' tone. |
| A3 | Enhance microphone | ☑ | Apple's on-device ML voice isolation (AUSoundIsolation) run offline, then levelled: noise floor −45 → −90 dB on a test clip, ~60× real time. DSP fallback. |
| E1–E3 | MP4, resolution, frame rate | ☑ | Hardware H.264/HEVC, quality presets by bits per pixel. |
| E4 | GIF | ☑ | Parallel NeuQuant palettes, delta frames (only changed tiles, transparent elsewhere), duplicate merging, loop option. |
| E5 | Targets | ✗ | File, clipboard, reveal in Finder. No upload/share links (no accounts). |
| E6 | Presets / size estimate | ☑ | |
| E7 | Progress, cancel, quick export | ☑ | "Export & copy" banner after recording. |
| E8 | Size-targeted export | ☑ | DaisyCap's "Fit to file size" (MP4) stays at or under the target, re-encoding once if the first pass overshoots. |
| P1 | Projects library | ☑ | `~/Movies/DaisyCap`, autosave. Editor-first window: no separate library or New-recording screen; recent recordings (with rendered thumbnails) are a popover off the window title, plus Open… and the projects folder. |
| P5 | Live source preview | ☑ | Recording setup shows the selected display, area or window live, filling the canvas under the Record button. Separate ScreenCaptureKit stream at 10 fps, 1280 px, cursor shown, DaisyCap's own windows excluded; restarts when the source changes (after 300 ms of no edits), stops while recording, when the window is hidden, minimised or covered, and when a recording is open. iPhone sources show no preview. The GUI selftest checks frames arrive and that it's off while recording. |
| P2 | Style presets | ☑ | Save/apply/delete, default preset for new recordings. |
| P3 | Editor sidebar + selection inspector | ☑ | |
| P4 | Menu bar | ☑ | Record display/window/area, pause, status with elapsed time. |
| P6 | Paid-tier badges and checkout link | ✕ | Removed on 2026-11-02. Nothing is locked; the watermark, end credit, reframe, batch, brand kit and share are plain settings for everyone. |
| O6 | End credit, reframe, social batch | ☑ | A dark card at the end where the daisy spins, slides aside and the words wipe in "Made in DaisyCap" (3.4 s, 2–6 s adjustable; on by default, and a plain toggle turns it off). "All three shapes" in the export sheet writes 16:9, square and vertical copies one after another, and with a forced shape the frame pans so the click or zoom target stays in view (`Style.auto_reframe`, clamped so the background never shows). Also: importing a video keeps its sound (the system-audio track points at the video's own track), and captions transcribe the audio track when there is no microphone. |
| O7 | Brand kit | ☑ | A logo (imported into the project) and two colours, used on the end credit and in place of the daisy mark; the credit's backdrop and accent follow the brand. Grab-bag fixes found on the way: captions and steps only draw when there are any. |
| S1 | Share links | ☑ | Export → Share link uploads the file and its poster (a JPEG of the frame at the playhead) to jaeswift.xyz and copies `jaeswift.xyz/daisycap/<handle>/<video>`. A handle is claimed once; the server keeps only a hash of its token, so only this Mac can upload to it or take a video down. Server code: `server/share/` (deployed to `/opt/daisycap-share`, routed by `daisycap-share.conf`). Limits: 1 GB a video, 5 GB a handle, 8 MB a poster. Free for now. |
| O8 | Blur brush | ☑ | A mask kind that is painted: drag on the preview and each dab is a soft circle, drawn as a blur so a stroke reads as one shape and follows the zoom. Brush size, blur strength and Clear strokes in the inspector; the timeline treats it like any mask (move/resize its time range). |
| P9 | What's new + demo project | ☑ | A "What's New" sheet shows once after an update (Help › What's New… reopens it), driven by the version in `Cargo.toml` and `settings.last_version`. "See what it can do…" on the start screen builds a demo project on first use — a synthetic recording dressed with cards, callouts, a mask, captions and a zoom — then opens it. |
| P8 | Import any video | ☑ | Drop an MP4/MOV (or "Import a video…") and it becomes an editable project: the file is hard-linked (copied if needed) into a package as the screen track, with a fresh project, one clip over its length and auto-zooms. Opened from the window or via Finder's Open-with. Recordings are unchanged; this just widens what the editor edits. |
| P7 | Feedback box | ☑ | Help › Send Feedback…, the menu-bar menu, and "Report this" on any error open a box: what happened, an optional email, and the diagnostics it will send with a preview, plus a copy fallback if the send fails. Posts to `jaeswift.xyz/api/feedback.php`, which appends to `/var/lib/daisycap-feedback/feedback.jsonl` and opens a GitHub issue (the token stays on the server). Unit-tested; a network test posts to the live endpoint. |

## Verification on this MacBook (M4 Pro, macOS 27)

`scripts/selftest.sh --screen` (2026-10-12): 13/13 pass.

- `cargo test` (116): engine, project, presets, renderer helpers, masks,
  device frames, typing detection, caption editing, transcription chunking,
  Whisper transcription of synthesized speech,
  hotkeys, timeline clamping and cuts, voice isolation, masked-content
  change detection, window picking, GIF quantisation, icon, the sound bank
  (every click and key flavour finite, unique and deterministic, keys
  distinct per key code), click/key/whoosh placement under cuts and speed,
  the daisy watermark drawing only the corner, a feedback report's diagnostics and
  JSON (and, with --ignored, a live post to the feedback endpoint), plus a
  headless UI smoke test driving every editor panel
  opening on hover and closing when the pointer leaves, and background tests
  that edit each Background setting through the editor, read the preview's
  pixels back and compare them (and the exported frame) before and after,
  and audio tests (track levels and the silence note, the playback mix
  following mutes while open, the exported MP4 carrying the tone, tone
  detection under louder sound).
- Synthetic package → MP4 and GIF.
- Background check (`daisycap check-background`): 22 edits (each wallpaper,
  blur on and off, gradient, its angle and colours, two colours, an image,
  image blur, padding, roundness, shadow) each visibly change a rendered
  frame; run on the synthetic package and on the real recording.
- Synthetic iPhone recording (1180×2556, kind = device) → phone frame → MP4.
- Scripted GUI run (`GIMBAL_GUI_SELFTEST=1`): launch (opens on recording
  setup) → area picker opens over the display (screenshotted) and is
  confirmed → window picker opens on every display, highlights a window
  (screenshotted) and is cancelled → record (test pattern) → pause/resume →
  stop → editor opens the new project → export → Back to recording setup →
  the recording heads the recents list → reopen it → quit.
- Real 5K display recording (Screen Recording granted to the terminal) →
  export, and the background check on it.
- Audio round trip (`--audio`, part of `--screen`): a helper app plays a
  1234 Hz tone → DaisyCap records it (5.7 s of 6 heard) → the editor plays the
  recording while a second recording listens (5.8 s heard; ×0.25 volume
  measured at −12.0 dB, mute silent, seek in sync) → the export carries it
  → a recording of another app's window hears it too → the microphone
  hears it through the speakers (reported only: depends on lid/headphones).
- Window resize: DaisyCap's own window resized to 1024×700 through the
  Accessibility API and read back. Masks, device frames and voice-over mixing checked on real
  footage with `daisycap frame` and ffmpeg.

UI layouts are checked with `daisycap snapshot`, which renders the real UI
headlessly to PNG (home, every inspector tab, zoom/mask selection, export
sheet, post-recording banner; light and dark).

## What you need to do (permissions)

The app bundle has its own privacy identity (`com.daisycap.app`), signed
with the Apple Development identity so grants survive rebuilds.

1. `scripts/build.sh`, then `open dist/DaisyCap.app`.
2. Press **Request access** on the home screen (or start a recording). Then in
   **System Settings › Privacy & Security**:
   - **Screen & System Audio Recording**: enable **DaisyCap** (needed for any
     screen, window or area recording and for system audio). Quit and reopen
     DaisyCap afterwards; macOS only applies this on relaunch.
   - **Microphone**: enable **DaisyCap** (prompted the first time a recording or
     voice-over uses the microphone).
   - **Camera**: enable **DaisyCap** (prompted the first time a recording uses
     the camera).
   - **Input Monitoring**: add/enable **DaisyCap** to record keyboard shortcuts
     for the shortcut overlay and typing speed-up (mouse and cursor logging
     need no permission).
   - **Speech Recognition**: allow when prompted by *Transcribe microphone*
     in the Captions tab.
   - **Accessibility**: enable **DaisyCap** to use *Resize* on the Window source
     (macOS asks the first time).
3. To try iPhone/iPad recording: connect the device with a cable, unlock it,
   tap *Trust*, then pick **iPhone** as the source (`daisycap devices` lists it).
4. Optional end-to-end check from Terminal (uses the terminal's own grants):
   `scripts/selftest.sh --screen` and `scripts/bench.sh`.

Already verified without your input: Screen Recording, Microphone and
Accessibility work from the terminal (real 5K recordings and exports, a
live mic track, window resizing above). The MacBook's lid was closed during
testing, so the built-in microphone delivered silence; to hear real speech
in a test, open the lid or use an external microphone. Camera and Speech
Recognition have not been granted on this Mac, so live camera capture and
transcription were exercised only through their code paths and test media.

## Log

- 2026-10-12: recording setup redesigned after Rec: the
  selected source live fills the canvas, with the camera in a round bubble;
  a right sidebar with SCENE / SCREEN / CAMERA / AUDIO / OPTIONS and recent
  recordings (Open a recording…); a bottom bar with a small record button,
  timer and live System / Mic meters. Camera and iPhone previews
  (AVCaptureSession, 640 px); system level from the preview stream, mic
  level from CoreAudio. Window fix: a saved window id that went stale
  (app relaunched) is found again by app and title, else cleared so Record
  asks; untitled windows the picker offers now show in the window list.
- 2026-10-11: engine core with tests.
- 2026-10-11: wgpu renderer, AVFoundation decode, VideoToolbox/GIF export,
  recorder with input logging; zero-copy export path.
- 2026-10-11: egui app (library, editor, recording widget, menu bar, hotkey),
  on-device captions, DaisyCap.app bundle with generated icon, self-tests,
  performance measurements.
- 2026-10-11: style presets, desktop-wallpaper background, typing speed-up,
  loop playback, caption editing, masks, area picker, hide icons/Dock,
  configurable shortcuts, device frames, voice-over, iOS device capture,
  menu bar sources, quick export, Finder document opening, GIF delta frames.
- 2026-10-11: "Data may be visible" warnings for masks (`daisycap check-masks`);
  resize the recorded window (`daisycap resize-window`); on-screen window
  picker covering every display; warning when the built-in mic is off with
  the lid closed; Whisper captions.
- 2026-10-12: audio fix (reported: recordings had no sound in the
  editor). Editor playback rebuilt on CoreAudio with a live mix; window
  recordings now hear every app; `daisycap record` no longer mutes a running
  DaisyCap; the Audio tab explains silent tracks and a mic that was off (his
  recordings had the mic off and, mostly, nothing playing); exports no longer
  leave `.sb-…` temporaries next to the file; `daisycap audio-check` and the
  selftest's audio round trip.
- 2026-10-11: new app icon (requested: no purple): a Big Sur
  superellipse tile in graphite fading to near-black, a white cursor
  gliding in on a tapered curved trail, and a small red record dot; a test
  checks it has no purple pixels.
- 2026-10-11: background settings fix (reported: no visible effect).
  Full-screen recordings defaulted to 0 % padding, which hides the
  background entirely; blur had nothing to soften on the smooth generated
  wallpapers; roundness and shadow were in recording pixels, so a third of
  the intended size on Retina recordings. Pixel-checked in tests and the
  selftest.
- 2026-10-11: native floating recording control bar.
- 2026-10-11: live preview of the selected source on the recording canvas.
- 2026-10-11: editor-first window: recording setup in the editor layout,
  recents popover off the title, compact Record + source menu in the
  editor's title bar; no separate library screen.
- 2026-10-11: "Fit to file size", Enhance voice (ML isolation + AGC),
  fixes from a full code review (preview audio lifecycle, timeline clamps,
  area bounds, voice-over/editor lifecycle, quit and restart behaviour).
- 2026-11-02: intro/outro cards (generated title and end cards on the project
  background, faded, with the daisy mark).
- 2026-11-02: callouts (arrows, boxes and numbered step pins; drawn with a new
  shader SDF, pinned in time, following the zoom).
- 2026-11-02: audio worth keeping (waveform under the timeline, one-click
  silence trim, per-slice fades; cuts fade their edges).
- 2026-11-02: speed ramps (eased speed on a clip) and dip-at-cuts transitions.
- 2026-11-02: import any video (drag an MP4/MOV in, or Import a video…).
- 2026-11-02: "What's New" sheet after an update (also Help menu) and a
  one-click demo project. Version bumped to 0.2.0.
- 2026-11-02: 0.2.1 — the What's New sheet also shows on the first launch
  after it shipped (a missing remembered version no longer counts as brand
  new); the demo's constant 440 Hz tone became three soft chimes.
- 2026-11-02: end credit: no glow (the credit uses a daisy without its amber
  halo; the corner mark and icon keep theirs); "Made in DaisyCap" wipes in
  beside the daisy as it slides; the pair is centred with the real font metrics
  (`src/fonts.rs`); a brand logo replaces the corner mark too.
- 2026-11-02: the end credit's beats scale with its length (a 2 s card still
  shows the word); one `timemap::for_project` builds every project's timeline
  (the editor's playback mix had left the credit out); credit checked in the
  editor preview, the MP4 (with sound) and the GIF.
- 2026-11-02: share links (`src/share.rs`, `src/app/sharewin.rs`, `server/share/`).
  Checked on the live server: claim, upload, poster, page, the video's bytes,
  a wrong token refused, a taken handle refused, delete, a 30 MB upload stored
  and served back at full size. `cargo test --test share -- --ignored` repeats
  the live part. Version 0.2.4.
- 2026-11-02: freehand blur brush (a painted mask kind).
- 2026-11-02: brand kit (logo, accent, credit background); caption styles and
  presets; the DaisyCap word is the app menu.
- 2026-11-02: 0.2.2/0.2.3 — env knobs renamed to DAISYCAP_* (GIMBAL_ still
  read); imported videos keep their sound and captions transcribe the audio
  track when there is no microphone; a dark end credit (daisy spins, then
  "Made in DaisyCap"); "All three shapes" social batch; auto-reframe so a
  forced shape follows the action.
- 2026-11-02: Shared videos are deleted an hour after upload. `server/share/cleanup.php` runs from cron every five minutes and gives the bytes back to the handle; `view.php` stops serving an expired video at once.
- 2026-11-02: Paid-tier badges, locked toggles and the checkout link removed. The watermark and end credit are plain toggles, on by default as before.
- 2026-11-02: watermark is a plain toggle, on by default;
  the daisy mark turns once every ~4 s like the splash; the timeline panel is
  sized to its tracks (and scrolls in a short window) so nothing is clipped,
  and the status note sits inside the preview.

- Live preview is Retina-sharp, as in Rec: screens are captured at native
  pixel size (`SCCaptureResolutionType::Best`, sRGB, 30 fps), Lanczos-scaled
  on the GPU (MPS) to exactly the canvas's pixels, and drawn 1:1,
  pixel-aligned, from the frame's IOSurface (no CPU copy or upload). Camera
  bubble at 720 px / 30 fps.

- Full screen fix (reported: the UI breaks in full screen). The
  traffic-light inset moved AppKit's full-screen titlebar views, leaving a
  stray band and line at the top; the toolbar kept 88 pt for traffic lights
  that full screen hides; and the preview excluded only the windows DaisyCap had
  at start, so the full-screen titlebar window showed up in it. Now the inset
  skips full screen, the toolbar margin follows it, the preview excludes
  DaisyCap by application, and when DaisyCap covers the display it would record the
  preview says so instead of showing a black void. Checked at min size,
  maximised, full screen and back.

- Export quality (reported: exports look soft). Exports default to
  Native: the recording's own pixels, unscaled (e.g. 4654×3200 for a
  4112×2658 Retina display with padding; up to 8192×4320), with 4K, 1440p,
  1080p and 720p presets. Above H.264's 4096×2304 limit the export uses HEVC
  (HEVC Main, auto level). Quality presets are VideoToolbox constant quality
  (Studio 0.95, Social 0.85, …) instead of bits-per-pixel, and ProRes 4444
  (.mov, 4:4:4) is offered for lossless-looking masters. Downscaled exports
  resample the screen with a Catmull-Rom filter stretched to the footprint
  instead of bilinear (which aliased text into moiré). Exports are tagged
  BT.709 primaries/matrix with the sRGB transfer function — what the pixels
  are — so QuickTime/Safari show them without a gamma shift. Recordings use
  constant-quality HEVC (0.95). Measured on a full-display fine-text
  recording (export frame vs the recording, Lanczos-scaled for presets):
  1080p 31.4 → 33.9 dB, 4K preset 37.2 → 39.3 dB (now at the 4:2:0 ceiling),
  default 1080p → native 48.3 dB / SSIM 0.992; ProRes 49.2 dB / 0.996.
  `tests/resample.rs` checks native is pixel exact and downscales don't alias.
