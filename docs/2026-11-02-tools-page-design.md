# Tools page: say what DaisyCap is

Status: approved on 2026-11-02
Scope: `tools/daisycap/`, the page at jaeswift.xyz/tools/daisycap. Nothing else changes.

## Why

The page opens on word cutting. Its headline is "Record your screen. Edit it by deleting words.", and word editing has its own section with a still image. The owner wants the page to say what DaisyCap is, with no single feature as the headline. Word cutting stays, as one of eight cards of equal size, with its animated replay.

## Goals

- A headline that says what the app is: "A screen recorder and editor for Mac."
- Eight feature cards of the same size. Two columns on desktop, one on phones.
- A picture on every card, all the same size. The word-cut card's picture is the existing animated replay.
- Every claim matches the code. The PR names the source file for each one.
- The download button, the version line and the first-launch note keep their wording.

## Non-goals

- No changes to the app, the download page in `site/`, the share server or the DMG.
- No deploy until the owner asks for one.
- No open-source claim. The repo is not public yet.

## Anti-goals

- No card that looks bigger or more important than the others.
- No claim the app can't back up. Captions get no "stays on your Mac" claim, because Apple's engine only forces on-device recognition where macOS supports it.
- No Pro wording, GitHub links, outside hosts, third-party scripts or new analytics.

## Constraints

- The page is static HTML, CSS and JavaScript. `server/tools-daisycap/deploy.sh` copies it to `/var/www/daisycap-tools` with tar, which never deletes files, so the removed image has to be deleted on the server by hand.
- Screenshots come from `daisycap synth`, never from a real desktop.
- The page sets `color-scheme: light` and has no dark theme. The dark check in AGENTS.md doesn't apply, and the PR says so.
- Copy uses no em dashes.

## Approaches

- A. Small icons on every card. Quick to build. The replay's moving track would then be the only picture in the grid, so that card would still stand out. Not chosen.
- B. A picture on every card, with the replay in its own picture slot. Chosen. Every card reads as one of eight. It needs seven new screenshots and makes the page longer.
- C. Keep the word-cut section and change only the headline. The smallest change, but word cutting keeps its own section, which is the headline the owner wants gone. Not chosen.

## The page

- Header: DaisyCap, then Features and Download. The Editing link goes.
- Hero: no eyebrow. The headline, the lead, two buttons (Download for macOS and See the features), the Background-tab screenshot, and the version line.
- Features: the heading "What is in the app", the eight cards, and one line under them: "Also in the app: blur masks, click and keyboard sounds, importing MP4 and MOV files, an end card, and a shortcut overlay."
- Removed: the "Edit by deleting words" section and its still image, the How it works steps, and the separate share section. Share becomes a card.
- Unchanged: the download section and the footer.

## Copy

Lead: "DaisyCap records your screen, adds zooms and captions, and exports an MP4 or GIF. Recordings stay on your Mac unless you share one."

| Card | Copy |
|---|---|
| Record | Capture a display, a window or an area, with system audio and your mic. |
| Zooms and callouts | Zooms follow your clicks. Callouts add arrows, boxes and numbered steps. |
| Delete words to cut | Select words in the captions and press Delete. The video cuts there, and the gap closes. |
| Captions | Whisper or Apple's speech engine transcribes the audio. Edit the lines, then save .srt or .vtt. |
| Backgrounds and frames | Wallpapers, gradients and padding, with laptop, phone and tablet frames. |
| Silence and speed | Cut silences, speed up typing, and ease a clip's speed in and out. |
| Export | Save an MP4 or a GIF, or export 16:9, square and vertical in one go. |
| Share | Upload an export and copy a link. Links last one hour, then the video is deleted. |

## Pictures

Seven screenshots, one for each card except the word-cut card: the recording setup, a zoom with a callout, the captions tab, the background tab, the audio tab, the export sheet and the share window. They come from the synthetic demo, at the card's 16:10 slot. Before the build, the snapshot command gets checked against each one. If one can't be produced, the owner hears about it before a stand-in is used.

## Verification

- Each claim is checked against `src/` or `scripts/`, and the PR names the file.
- Renders at 1440 px and 390 px, light only. The replay loops, and with reduced motion it shows its final state.
- `git diff --check` passes, the text has no em dashes, and the only outside host is jaeswift.xyz.
- Before and after images go in the PR.

## Deploy

After the owner says so: run `server/tools-daisycap/deploy.sh root@81.0.219.227`, then delete `img/words.webp` on the server.

## Left to the builder

The snapshot commands and crops, the card grid and picture slot CSS, the header change, and the claims table in the PR body.

## Calls I made

- The headline says "editor", which names what the app does.
- The eyebrow, How it works and the separate share section go, because the cards cover them.
- Only `tools/daisycap/` changes. The download page in `site/` keeps its own copy.
