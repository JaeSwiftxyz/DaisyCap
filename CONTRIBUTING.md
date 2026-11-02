# Contributing to DaisyCap

DaisyCap is a native Mac screen recorder in Rust. It records a display, a window, an area or an iPhone, and exports MP4 or GIF. The aim is a fast, private recorder: recordings stay on the Mac unless you share one.

Contributions are welcome. One rule matters above the rest: the app is free and nothing in it is locked. Don't add a paid tier, a licence key or a Pro badge. The watermark and end credit stay plain toggles.

## Setup

You need macOS 15 or later, Rust with the 2024 edition, Xcode and CMake. The app itself runs on macOS 14.

```sh
brew install cmake
. scripts/env.sh
cargo build
scripts/build.sh
open dist/DaisyCap.app
```

Build output goes to `~/.cache/cargo-target/daisycap`, never into the repo. `scripts/env.sh` sets `SDKROOT` for whisper.cpp, which captions use.

## Checks

Run these before you open a PR. CI doesn't run them yet, so the PR says what you ran.

For the app:

```sh
. scripts/env.sh && cargo test
scripts/build.sh
```

Run `scripts/selftest.sh` when you change capture, export, audio or the editor. It plays a quiet test tone, and `--screen` records the display.

For the download page in `site/`:

```sh
cd site
npm install
npm run check
npm run build
```

Then load the page and look at it at desktop width and at 390 px, in light and dark.

## Style

Match the code around you. No new dependencies without a reason in the PR. Keep a change to the job at hand; put unrelated fixes in their own PR.

Commit subjects follow `type(scope): lowercase summary`, for example `feat(app): keyboard shortcut overlay` or `fix(export): keep the last frame`.

The pages claim only what the app ships. Check a feature in `src/` before a page mentions it. Screenshots come from DaisyCap's own synthetic recordings (`daisycap synth`), never a real desktop, and the only outside host is jaeswift.xyz. Page copy uses no em dashes.

## Pull requests

The repo lives on Gitea at git.jaeswift.xyz. `gh` only reaches GitHub, so use the Gitea web UI or its API. Branch from `main`, open the PR against `main`, and don't merge your own PR unless it's agreed first.

Say what changed, how you tested it, and what is still unproven. Add before and after screenshots for anything visible.

## Agent workflow

`AGENTS.md` is for AI coding agents. The workflow skills it mentions (before-and-after, greploop, unslop and others) live on the `agent-notes` branch, so `main` stays readable. Add that branch to a checkout when you want them:

```sh
git fetch origin agent-notes
git checkout agent-notes -- .agents .claude
```

`.claude/skills` is a set of symlinks into `.agents/skills`.

## Licence

Contributions are accepted under MIT OR Apache-2.0. You keep the copyright in what you write.

Send security reports to the address in the git commits, or open a private issue if the repo supports it, rather than a public PR.
