# Tools page: say what DaisyCap is Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `tools/daisycap/` open with what DaisyCap is, with eight equal feature cards and the word-cut replay as one of them.

**Architecture:** Static HTML and CSS, with the existing `app.js` unchanged. The cards form a two-column grid, and each has a 16:10 picture slot. Seven slots hold synthetic-demo screenshots. The word-cut slot holds the existing animated replay.

**Tech Stack:** HTML, CSS, vanilla JavaScript, Chrome headless for renders, the `daisycap` CLI (`synth`, `snapshot`), cwebp and ImageMagick.

**Spec:** `docs/2026-11-02-tools-page-design.md`

## Global Constraints

- Work only in `~/Desktop/daisycap-worktrees/tools-what-it-is-1102h`, on `agent/tools-what-it-is-1102h`. Never commit to `main`.
- Only `tools/daisycap/` and `docs/` change.
- Headline, lead, card titles and card copy come verbatim from the spec's Copy section.
- No em dashes or en dashes in changed files.
- The only outside host is `jaeswift.xyz`. Add no scripts, fonts, analytics or tracking.
- Keep `color-scheme: light`. The page has no dark theme.
- Pictures come from `daisycap synth` and `daisycap snapshot` only, never a real desktop.
- No claim that captions stay on the Mac. No Pro wording. No GitHub links.
- Commit as JaeSwift, with `type(scope): lowercase summary` subjects. Run `/unslop` over commit messages and the PR text.

## Review Focus

1. A 390 px phone shows one column with no sideways scroll, and the replay's words wrap inside their slot. Tested in Task 5.
2. With JavaScript off, all eight card titles and the replay's cut state show. Tested in Task 5.
3. With reduced motion on, every card shows and the replay rests in its cut state. Tested in Task 5.
4. A slow or missing picture leaves a box of the right size. Every image has `width` and `height`. Tested in Task 2 and Task 5.
5. Each nav link reaches an id on the page. Tested in Task 2.

---

## Claims map

Each claim the page makes, and the source that backs it. Task 5 Step 2 rechecks every row, and the PR body repeats the table.

| Claim | Source |
|---|---|
| Record a display, a window or an area, with system audio and a mic | PROGRESS.md R1 to R5 |
| Zooms follow your clicks | `src/engine/zoom.rs`, `auto_zooms` |
| Callouts add arrows, boxes and numbered steps | PROGRESS.md O4 |
| Delete words to cut the video | `src/engine/words.rs` (cut spans) |
| Whisper or Apple's speech engine transcribes the audio | `src/transcribe.rs`, `enum Engine` |
| Edit the caption lines, save .srt or .vtt | `src/engine/captions.rs`, `SubFormat::Srt` and `Vtt`; PROGRESS.md O2 |
| Wallpapers, gradients, padding, laptop, phone and tablet frames | `src/project/model.rs`, `enum Background` and `enum DeviceFrame`; PROGRESS.md S1 to S8 |
| Cut silences, speed up typing, ease a clip's speed | PROGRESS.md A5, T8, O5 |
| Save an MP4 or a GIF | `src/project/model.rs`, `enum ExportFormat { Mp4, Gif }` |
| Export 16:9, square and vertical in one go | `src/app/editor.rs`, "All three shapes"; `Aspect::PRESETS` |
| Upload an export and copy a link; links last one hour | `src/share.rs`; `server/share/cleanup.php`; `server/share/view.php` |
| Blur masks | `src/project/model.rs`, `enum MaskKind`; PROGRESS.md T9 |
| Click and keyboard sounds | PROGRESS.md C10 |
| Import MP4 and MOV files | PROGRESS.md P8 |
| End card | PROGRESS.md O6 |
| Shortcut overlay | PROGRESS.md O1 |
| Recordings stay on your Mac unless you share one | PROGRESS.md share row: upload only from Export, then Share link |
| Version 0.2.4, macOS 14 or later | `Cargo.toml` line 3; `support/Info.plist`, `LSMinimumSystemVersion` |
| Apple Silicon and Intel | `scripts/package-dmg.sh`, `UNIVERSAL=1` |
| Not notarized; no automatic update check | `scripts/package-dmg.sh` header; `src/app/updater.rs` |

---

## Task 0: Capture the current page

**Files:** none changed. Evidence goes to `.artifacts/tools-what-it-is/`, which is git-ignored.

- [ ] **Step 1: Serve the current page on a free port.**

```bash
cd ~/Desktop/daisycap-worktrees/tools-what-it-is-1102h/tools/daisycap
mkdir -p ../../.artifacts/tools-what-it-is
PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
python3 -m http.server "$PORT" --bind 127.0.0.1   # run in the background
lsof -nP -iTCP:"$PORT" -sTCP:LISTEN               # expect your python3 process
```

- [ ] **Step 2: Render at desktop width.** Use `--force-prefers-reduced-motion`, so every reveal shows, and a window tall enough for the whole page. Check the bottom edge of the image.

```bash
CHROME="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
"$CHROME" --headless --disable-gpu --hide-scrollbars --force-prefers-reduced-motion --window-size=1440,2600 --screenshot=../../.artifacts/tools-what-it-is/before-1440.png "http://127.0.0.1:$PORT/"
```

- [ ] **Step 3: Render at phone width.** Chrome's window can't go below about 500 px, so put the page in a 390 px iframe and crop.

```bash
WRAP=$(mktemp -d "/private/var/folders/h5/91x9q25n3sz77dq3cl9vg8kr0000gn/T/opencode/daisycap-phone.XXXXXX")
printf '<iframe src="http://127.0.0.1:%s/" width="390" height="2600" style="border:0"></iframe>' "$PORT" > "$WRAP/phone.html"
"$CHROME" --headless --disable-gpu --hide-scrollbars --force-prefers-reduced-motion --window-size=500,2600 --screenshot="$WRAP/phone.png" "file://$WRAP/phone.html"
magick "$WRAP/phone.png" -crop 390x2600+0+0 +repage ../../.artifacts/tools-what-it-is/before-390.png
```

- [ ] **Step 4: Stop the server.** Kill the `python3` process found in Step 1.

---

## Task 1: Make the pictures

**Files:**
- Create: `tools/daisycap/img/record.webp`, `zoom.webp`, `captions.webp`, `backgrounds.webp`, `audio.webp`, `export.webp`, `share.webp`
- Replace: `tools/daisycap/img/hero.webp`, `tools/daisycap/img/hero.jpg`

**Interfaces:**
- Produces: seven WebP files and the hero pair, each 1200 by 760 pixels. Task 2 uses these names exactly.

- [ ] **Step 1: Build the CLI in the clone.** `. scripts/env.sh && cargo build` puts the binary at `BIN=$HOME/.cache/cargo-target/daisycap/debug/daisycap`, which the later steps use. Build output stays in the clone, not the main checkout.
- [ ] **Step 2: Read the options.** Run `"$BIN" synth --help` and `"$BIN" snapshot --help`. Note the tab names and view flags. Pick the synth options that give the demo a zoom, a callout and captions.
- [ ] **Step 3: Make the demo project** in a `mktemp -d` folder, using those options.
- [ ] **Step 4: Snapshot each picture** with `--light --size 1200x760`:
  - hero: the Background tab
  - record: the recording setup view
  - zoom: the editor with a zoom and a callout on screen
  - captions: the Captions tab
  - backgrounds: the Background tab with a device frame chosen
  - audio: the Audio tab
  - export: the export sheet
  - share: the share window (`--share`)
- [ ] **Step 5: Convert.** `cwebp -q 85 <png> -o tools/daisycap/img/<name>.webp` for each card. For the hero, also `magick <png> -quality 85 tools/daisycap/img/hero.jpg`.
- [ ] **Step 6: Check sizes.** `magick identify -format '%f %wx%h %b\n' tools/daisycap/img/*.webp` shows 1200x760 for every file, each under 200 KB.
- [ ] **Step 7: Look at each picture.** Confirm it shows what its card names. Re-snapshot any that don't.
- [ ] **Step 8: Commit.** `feat(tools): pictures for the feature cards`

**Gate:** If a view can't be produced, stop and ask the owner. Don't use a stand-in (design doc, Pictures section).

---

## Task 2: Rewrite the page markup

**Files:** Modify `tools/daisycap/index.html`.

**Interfaces:**
- Keeps the hooks `app.js` reads: `[data-reveal]`, `[data-delay]`, `[data-replay]`, `.track`, `.gap`, and `.replay-btn` inside `[data-replay]`.
- Produces: `.card`, `.card-picture` (a `figure` for pictures, a `div` for the replay), `.replay` inside its `.card-picture`, and `.also`.

- [ ] **Step 1: Write the check script** to `.artifacts/tools-what-it-is/check-markup.sh`. It stays out of the commit.

```bash
cd tools/daisycap
test "$(grep -c '<article class="card' index.html)" -eq 8 || echo "FAIL: eight cards"
test "$(grep -c 'class="card-picture' index.html)" -eq 8 || echo "FAIL: eight picture slots"
test "$(grep -c 'data-replay' index.html)" -eq 1 || echo "FAIL: one replay"
grep -q 'A screen recorder and editor for Mac\.' index.html || echo "FAIL: headline"
grep -Eq 'Editing|id="edit"|id="share"|id="how"|class="still"|words\.webp|class="eyebrow"' index.html && echo "FAIL: retired part still present"
grep -n $'\xe2\x80\x94\|\xe2\x80\x93' index.html && echo "FAIL: dash"
for id in $(grep -Eo 'href="#[a-z]+"' index.html | sed 's/href="#//; s/"//' | sort -u); do grep -q "id=\"$id\"" index.html || echo "FAIL: no id for #$id"; done
grep '<img' index.html | grep -vq 'width=' && echo "FAIL: image without width"
```

- [ ] **Step 2: Run it on the current page.** Expect several `FAIL` lines.
- [ ] **Step 3: Head.** Title: `DaisyCap: a screen recorder and editor for Mac`. Meta description: "DaisyCap is a screen recorder and editor for Mac. Record, add zooms and captions, export an MP4 or GIF, and share a link." `og:title` matches the title. `og:description`: "A screen recorder and editor for Mac, with zooms, captions, backgrounds and share links." Keep the canonical, `og:image` and `og:url`.
- [ ] **Step 4: Header nav.** Features (`#features`) and Download (`#download`). Remove Editing.
- [ ] **Step 5: Hero.** Remove the eyebrow paragraph. The `h1` is the headline and the lead is the spec's lead. Buttons: "Download for macOS" (same href) and "See the features" (`#features`, class `btn ghost`). Keep the version line. The image is `img/hero.webp` with `width="1200" height="760"` and one sentence of alt text.
- [ ] **Step 6: Features.** `section.features#features` holds an `h2` reading "What is in the app" and a `div.grid`. The grid holds eight `article.card.reveal` items, in this order: Record, Zooms and callouts, Delete words to cut, Captions, Backgrounds and frames, Silence and speed, Export, Share. Delays are 0, 40, 80, 120, 160, 200, 240 and 280 ms, as the current page uses.
  - Seven cards hold a `figure.card-picture` with an `img` (names: record, zoom, captions, backgrounds, audio, export, share), `width="1200" height="760" loading="lazy" decoding="async"`, and alt text that doesn't repeat the card title.
  - The word card holds `div.card-picture` wrapping `div.replay[data-replay]`. That contains `p.words` (`aria-hidden="true"`), `div.track` with `span.clip.keep-a`, `span.clip.gap` and `span.clip.keep-b`, and `button.btn.ghost.small.replay-btn`. Drop `.replay-foot` and `.caption`.
  - Each card then has an `h3` and a `p` with the spec's copy.
- [ ] **Step 7: "Also in the app".** After the grid, a `p.also` with the line from the spec.
- [ ] **Step 8: Remove sections.** Delete `section#edit`, `section#share` and `section#how`. Keep `section.download#download` and the footer.
- [ ] **Step 9: Run the check script.** Expect no output.
- [ ] **Step 10: Commit.** `feat(tools): the page opens on what the app is`

---

## Task 3: Restyle for the card grid

**Files:** Modify `tools/daisycap/styles.css`.

- [ ] **Step 1: Delete rules for removed markup.** `.eyebrow`, `.intro`, `.still` (with its `img` and `figcaption`), `.share-grid` (with its `p`), `.privacy`, `.steps` (with `li` and `p`), `.num`, `.rule` (with its `.js` and reduced-motion lines), `.caption`, `.replay-foot`, and the `.share-grid` and `.steps` lines in the wide-screen block.
- [ ] **Step 2: Fix the phone nav.** Delete `.top nav a:first-child { display: none; }` from the 520 px block. It would hide Features on phones.
- [ ] **Step 3: Grid.** One column by default. At `min-width: 700px`, `grid-template-columns: repeat(2, minmax(0, 1fr))` with an 18 px gap. Remove the `auto-fill` rule.
- [ ] **Step 4: Picture slot.**
  - `.card { display: flex; flex-direction: column; }`
  - `.card-picture { position: relative; aspect-ratio: 16 / 10; margin: 0 0 16px; overflow: hidden; border-radius: 12px; background: var(--lane); }`
  - `.card-picture img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }`
- [ ] **Step 5: Replay in its slot.** Replace the existing `.replay` rule with `position: absolute; inset: 0; margin: 0; padding: clamp(14px, 2.6vw, 24px); border: 0; border-radius: 0; background: var(--bg); display: flex; flex-direction: column; justify-content: center;`. Add `.replay .words { font-size: clamp(1rem, 2.2vw, 1.25rem); }`, `.replay .track { margin-top: 18px; height: 40px; }` and `.replay .replay-btn { position: absolute; right: 12px; bottom: 12px; }`.
- [ ] **Step 6: The "also" line.** `.also { margin: 28px 0 0; color: var(--muted); font-size: 15.5px; }`
- [ ] **Step 7: Keep the rest.** The reveal rules, the replay motion rules, the reduced-motion block (minus the deleted selectors), and the download and footer rules stay.
- [ ] **Step 8: Check whitespace.** `git diff --check` prints nothing.
- [ ] **Step 9: Commit.** `feat(tools): card grid styles`

---

## Task 4: Remove the retired picture

**Files:** Delete `tools/daisycap/img/words.webp`.

- [ ] **Step 1:** `git rm tools/daisycap/img/words.webp`, then `grep -rn 'words.webp' tools/daisycap` prints nothing.
- [ ] **Step 2: Commit.** `chore(tools): remove the still the page no longer uses`

---

## Task 5: Verify and collect evidence

Follow `/evidence-driven-testing` for the evidence format. Evidence goes to `.artifacts/tools-what-it-is/`.

- [ ] **Step 1: Markup checks.** Run the Task 2 check script. Expect no output.
- [ ] **Step 2: Claims.** Walk the claims map and open each named source. A mismatch changes the copy and the map together.
- [ ] **Step 3: Desktop, reduced motion.** Render `after-1440.png` as in Task 0 Step 2. Read it and confirm: the hero, eight equal cards in two columns, the replay in its slot, the "Also" line, the download band and the footer.
- [ ] **Step 4: Phone.** Render `after-390.png` as in Task 0 Step 3. Confirm one column, Features and Download in the nav, replay words inside their slot, and nothing cut off on the right.
- [ ] **Step 5: JavaScript off.** Render at 1440 px with `--disable-javascript`. Confirm all eight titles and the replay in its cut state. `--dump-dom --disable-javascript` should contain eight `<h3>` titles.
- [ ] **Step 6: Console.** Render with motion on and `--enable-logging=stderr --v=0`, then `grep -i 'uncaught\|error' console.log` prints nothing.
- [ ] **Step 7: Images.** `magick identify` reports 1200x760 for every picture. Every `img` `src` exists in `img/`.
- [ ] **Step 8: Anti-goals.** `grep -rn $'\xe2\x80\x94\|\xe2\x80\x93' tools/daisycap` and `grep -rnw 'Pro' tools/daisycap` print nothing. `grep -Eoh 'https?://[^" )]+' tools/daisycap/*.html tools/daisycap/*.css tools/daisycap/*.js | grep -v jaeswift.xyz` prints nothing.
- [ ] **Step 9: Before and after.** Read `.agents/skills/before-and-after/SKILL.md` and build the table from the before and after images at both widths. The upload target is in Calls below.

---

## Task 6: Rebase, push and open the PR

- [ ] **Step 1: Rebase.** `git fetch origin && git rebase origin/main`. Rerun Task 5 Steps 1, 8 and 9 on the rebased tree.
- [ ] **Step 2: Push.** `git push -u origin agent/tools-what-it-is-1102h`
- [ ] **Step 3: Open the PR** with `gh pr create`, from `agent/tools-what-it-is-1102h` into `main`. Title: `feat(tools): the page says what DaisyCap is`.
- [ ] **Step 4: PR body,** in this order:
  - What changed
  - Claims: the claims map with sources
  - Evidence: the before and after table, and the image paths
  - Checks: the outputs from Task 5
  - Risks and follow-ups
- [ ] **Step 5: Risks to state.** `img/words.webp` stays on the server until deploy removes it, because the tar copy never deletes. The page has no dark theme, so the dark check doesn't apply. `/greploop` didn't run in that session, because the repo was on Gitea then.
- [ ] **Step 6: Follow-ups, as separate PRs.** PROGRESS.md still has Pro wording in several rows. `site/src/lib/site.ts` says version 0.1.0.
- [ ] **Step 7:** Run `/unslop` over the title and body before posting. Don't merge.

---

## Task 7: Deploy (only when the owner says so)

Don't run this task until the owner asks for a deploy.

- [ ] **Step 1: Publish.** From the clone, run `server/tools-daisycap/deploy.sh root@81.0.219.227`, using the SSH shim from earlier. The password comes from the environment, never from the repo.
- [ ] **Step 2: Remove the retired picture on the server.** `ssh root@81.0.219.227 'rm -f /var/www/daisycap-tools/img/words.webp'`
- [ ] **Step 3: Check the live page.** `curl -s https://jaeswift.xyz/tools/daisycap/ | grep -c 'A screen recorder and editor for Mac.'` prints 1. `words.webp` returns 404. Each of the seven card pictures and the hero returns 200.
- [ ] **Step 4: Compare.** Render the live page at 1440 px and compare it with `after-1440.png`.

---

## Calls

- The meta title and description are new, written from the spec's copy.
- The replay caption goes, since the card copy explains the cut.
- Before and after images use the default upload target (0x0.st), which is public. AGENTS.md allows that for the download page, and this page shows only public content. The owner can object before Task 5 Step 9.
