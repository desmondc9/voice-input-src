# Linux-Standalone Restructure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restructure the repository so its top-level identity is "Wayland-native voice-input for Linux" rather than "a fork of yetone/voice-input that contains a Linux port in a subdirectory."

**Architecture:** Five small commits, in order: (1) remove macOS-era artifacts, (2) flatten `linux/` to repo root via `git mv`, (3) update CI workflow paths, (4) change LICENSE author, (5) rewrite root `README.md` + `CLAUDE.md`. Each commit independently buildable. Between commit 2 (flatten) and commit 3 (path fix), the release workflow is broken — do not push tags in that window.

**Tech Stack:** Pure git operations + markdown rewrites + one YAML edit. No Rust code changes.

**Spec reference:** `docs/superpowers/specs/2026-05-19-linux-standalone-restructure-design.md` (commit `a287cc2`)

---

## Pre-flight

Verify before starting:

- On branch `main`, working tree clean
- `git log --oneline -1` shows `a287cc2 docs(spec): design for standalone-Linux repo restructure`
- v0.1.1 release is published (so we are not mid-release)
- `linux/` directory currently exists with `Cargo.toml`, `Cargo.lock`, `src/`, `tests/`, `scripts/`, `README.md`, `.gitignore`
- `dist/` git submodule is initialised (`git submodule status` shows a commit hash)
- `cd linux && cargo build --release` succeeds locally (we will rerun this after each commit)

```bash
cd /home/desmond/Repos/voice-input-src
git status                        # expect: clean working tree
git log --oneline -1              # expect: a287cc2
git submodule status              # expect: one line for dist/
ls linux/                         # expect: Cargo.toml Cargo.lock README.md scripts src target tests .gitignore
```

---

## File structure (after all 5 commits)

```
voice-input-src/
├── .github/workflows/release.yml      # paths updated (commit 3)
├── .gitignore                          # merged with linux/.gitignore content (commit 2)
├── CHANGELOG.md                        # unchanged
├── CLAUDE.md                           # rewritten (commit 5)
├── Cargo.lock                          # moved from linux/ (commit 2)
├── Cargo.toml                          # moved from linux/ (commit 2)
├── LICENSE                             # author changed to Desmond Chen (commit 4)
├── README.md                           # was linux/README.md; surgical edits (commit 5)
├── docs/superpowers/
│   ├── plans/
│   │   └── 2026-05-19-linux-standalone-restructure.md  ← this file
│   └── specs/
│       └── 2026-05-19-linux-standalone-restructure-design.md
├── plans/                              # unchanged (historical Linux phase plans)
│   ├── 2026-05-15-voice-input-linux/
│   └── 2026-05-18-deb-release/
├── scripts/                            # moved from linux/scripts/ (commit 2)
│   ├── install-autostart.sh
│   └── install-ydotool.sh
├── src/                                # moved from linux/src/ (commit 2)
└── tests/                              # moved from linux/tests/ (commit 2)
```

Files **removed** by this plan:

- `dist/` (the whole submodule, plus `.git/modules/dist/`)
- `README_CN.md` (Chinese version of the macOS prompt)
- `.gitmodules` (only entry was `dist`)
- `linux/` (entire directory after move)

The old `README.md` (macOS prompt, English) is removed in commit 1 and then a different file (the former `linux/README.md`) takes that path in commit 2.

---

## Task 1: Remove macOS-era artifacts

**Files:**
- Delete: `dist/` (submodule), `.gitmodules`, `README.md` (old macOS prompt), `README_CN.md`
- Untrack: `input_test.txt` (stays in working tree, ignored)
- Delete from `.git/`: `.git/modules/dist/`

- [ ] **Step 1: Confirm starting state**

```bash
git status
git submodule status
```

Expected: working tree clean. Submodule status shows one line for `dist/`.

- [ ] **Step 2: Deinit and remove the `dist/` submodule**

```bash
git submodule deinit -f dist
git rm -f dist
rm -rf .git/modules/dist
```

Expected: `dist/` directory gone from working tree, `.git/modules/dist/` gone.

- [ ] **Step 3: Remove `.gitmodules`**

```bash
git rm -f .gitmodules
```

Expected: `.gitmodules` gone. The previous step's `git rm dist` may have already emptied it; this step makes the deletion explicit and complete.

- [ ] **Step 4: Remove the macOS-prompt READMEs**

```bash
git rm README.md README_CN.md
```

Expected: both files gone from working tree and staged for deletion.

- [ ] **Step 5: Untrack `input_test.txt`**

`input_test.txt` is already listed in `.gitignore` but is still tracked from before the ignore rule existed.

```bash
git rm --cached input_test.txt
```

Expected: file removed from index but still present in working tree.

- [ ] **Step 6: Verify the staged state**

```bash
git status --short
```

Expected output (order may vary):

```
D  .gitmodules
D  README.md
D  README_CN.md
D  dist
D  input_test.txt
```

Verify the Linux project is still buildable:

```bash
cd linux && cargo check && cd ..
```

Expected: `Finished` (the `linux/` subdirectory is untouched in this commit).

- [ ] **Step 7: Commit**

```bash
git commit -m "$(cat <<'EOF'
chore(repo): remove macOS-era artifacts

Drops the yetone/voice-input macOS leftovers that no longer reflect
what this repo builds:

- dist/ submodule (yetone/voice-input-dist Swift package)
- README.md / README_CN.md (the original Claude prompt that
  generated the macOS app)
- .gitmodules (only entry was dist/)
- input_test.txt is untracked but kept locally (already in .gitignore)

Next commit flattens linux/ to the repo root. After that, a new
README and CLAUDE.md replace the macOS-focused documentation.
EOF
)"
```

- [ ] **Step 8: Verify commit**

```bash
git log --oneline -1
git show --stat HEAD
```

Expected: commit message starts with `chore(repo): remove macOS-era artifacts`. Stat shows 5 deletions (plus submodule entry).

---

## Task 2: Flatten `linux/` to the repo root

**Files:**
- Move: `linux/Cargo.toml`, `linux/Cargo.lock`, `linux/src/`, `linux/tests/`, `linux/scripts/`, `linux/README.md` → repo root
- Merge: `linux/.gitignore` content into root `.gitignore`, then delete `linux/.gitignore`
- Delete: `linux/` directory entirely (after move, including untracked `linux/target/`)

- [ ] **Step 1: Confirm Task 1 is committed and tree is clean**

```bash
git status
git log --oneline -1
```

Expected: clean tree; HEAD is `chore(repo): remove macOS-era artifacts`.

- [ ] **Step 2: Move the four tracked top-level files into the repo root**

```bash
git mv linux/Cargo.toml Cargo.toml
git mv linux/Cargo.lock Cargo.lock
git mv linux/README.md README.md
```

Expected: each `git mv` succeeds without output. `README.md` exists at repo root after the third command (the macOS-prompt README was deleted in Task 1, so there is no collision).

- [ ] **Step 3: Move the three tracked directories into the repo root**

```bash
git mv linux/src src
git mv linux/tests tests
git mv linux/scripts scripts
```

Expected: each succeeds. After this, `linux/` contains only `.gitignore` (tracked) and `target/` (untracked build output).

- [ ] **Step 4: Merge unique entries from `linux/.gitignore` into root `.gitignore`**

`linux/.gitignore` currently contains:

```
/target
*.swp
*.bak
.DS_Store
```

The root `.gitignore` already has `target/` (which matches the soon-to-be-at-root `target/`). The other three lines have no equivalent at root.

Append the three unique entries to root `.gitignore`. The file should end up as:

```
# Cargo build output
target/

# Built Debian packages
*.deb

# Local test / scratch files
input_test.txt

# Claude Code local state
.claude/

# Editor / OS junk
*.swp
*.bak
.DS_Store
```

- [ ] **Step 5: Remove `linux/.gitignore`**

```bash
git rm linux/.gitignore
```

- [ ] **Step 6: Verify the staged state**

```bash
git status --short
```

Expected output (renames shown by git as `R  old -> new`; exact format depends on git version):

```
M  .gitignore
R  linux/Cargo.lock -> Cargo.lock
R  linux/Cargo.toml -> Cargo.toml
R  linux/README.md -> README.md
D  linux/.gitignore
R  linux/scripts/install-autostart.sh -> scripts/install-autostart.sh
R  linux/scripts/install-ydotool.sh -> scripts/install-ydotool.sh
R  linux/src/<many files> -> src/<many files>
R  linux/tests/<many files> -> tests/<many files>
```

(The `R` lines for `src/` and `tests/` may break out into individual files — that is normal; git decides whether to display "rename of directory" or "rename of individual files" based on similarity scoring.)

- [ ] **Step 7: Verify the build from the new root**

```bash
cargo check
```

Expected: `Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs`. No errors. Cargo creates a new `target/` at the repo root.

- [ ] **Step 7b: Defensive check — any tracked files still under `linux/`?**

```bash
git ls-files linux/
```

Expected: **empty output**. If anything prints (e.g. an `assets/` directory or a stray dotfile that this plan did not anticipate), `git mv` it to the corresponding root path before continuing, then re-run this check until it is empty.

- [ ] **Step 8: Remove the now-empty `linux/` directory and stray build output**

After all `git mv` operations and step 5, `linux/` still exists on disk because it contains the untracked `linux/target/` (old build output) and any other untracked files. Clean it up:

```bash
rm -rf linux/
```

Expected: `ls linux/` reports "No such file or directory" afterwards. This does not affect the git index — `linux/target/` was untracked.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "$(cat <<'EOF'
refactor(repo): flatten linux/ to repo root

The repository was structured as if it might one day host multiple
platform ports, with the Linux Rust project in linux/. Since the
macOS side was removed in the previous commit, the linux/ wrapper
no longer serves any purpose — it just forced every command, path,
and reference to carry an unnecessary prefix.

This commit is a pure git mv: Cargo.toml, Cargo.lock, src/, tests/,
scripts/, README.md all move up one level. linux/.gitignore is
merged into the root .gitignore (gaining *.swp, *.bak, .DS_Store
entries the root file did not previously have). No content changes.

The release workflow still references linux/ paths and will be
broken until the next commit fixes it. Do not push tags in between.
EOF
)"
```

- [ ] **Step 10: Verify commit + build from new root**

```bash
git log --oneline -2
cargo build --release 2>&1 | tail -5
```

Expected: top commit is the flatten; `cargo build --release` shows `Finished `release` profile`.

---

## Task 3: Update CI workflow paths after flatten

**Files:**
- Modify: `.github/workflows/release.yml` (4 hunks)

- [ ] **Step 1: Inspect current workflow**

```bash
grep -n 'linux\|working-directory' .github/workflows/release.yml
```

Expected: you should see two `working-directory: linux` lines and two `path: linux/target/...` lines.

- [ ] **Step 2: Edit the `Build CPU .deb` step — remove `working-directory: linux`**

In `.github/workflows/release.yml`, find:

```yaml
      - name: Build CPU .deb
        working-directory: linux
        run: cargo deb
```

Replace with:

```yaml
      - name: Build CPU .deb
        run: cargo deb
```

- [ ] **Step 3: Edit the `deb-cpu` artifact upload path**

Find:

```yaml
      - uses: actions/upload-artifact@v4
        with:
          name: deb-cpu
          path: linux/target/debian/voice-input_*.deb
```

Replace the `path:` value:

```yaml
      - uses: actions/upload-artifact@v4
        with:
          name: deb-cpu
          path: target/debian/voice-input_*.deb
```

- [ ] **Step 4: Edit the `Build CUDA .deb` step — remove `working-directory: linux`**

Find:

```yaml
      - name: Build CUDA .deb
        working-directory: linux
        # whisper.cpp defaults CMAKE_CUDA_ARCHITECTURES=native; on a GPU-less
        # CI runner that falls back to nvcc's ancient default (sm_52, Maxwell),
        # producing a binary that crashes on any modern card with
        # `CUDA kernel mul_mat_vec has no device code compatible with CUDA arch N`.
        # Target a fat list covering Turing -> Hopper so the .deb runs on every
        # relevant NVIDIA GPU (T4, RTX 20/30/40 series, A100, H100).
        env:
          CMAKE_CUDA_ARCHITECTURES: "75;80;86;89;90"
        run: cargo deb --variant cuda
```

Replace with the same block minus the `working-directory: linux` line:

```yaml
      - name: Build CUDA .deb
        # whisper.cpp defaults CMAKE_CUDA_ARCHITECTURES=native; on a GPU-less
        # CI runner that falls back to nvcc's ancient default (sm_52, Maxwell),
        # producing a binary that crashes on any modern card with
        # `CUDA kernel mul_mat_vec has no device code compatible with CUDA arch N`.
        # Target a fat list covering Turing -> Hopper so the .deb runs on every
        # relevant NVIDIA GPU (T4, RTX 20/30/40 series, A100, H100).
        env:
          CMAKE_CUDA_ARCHITECTURES: "75;80;86;89;90"
        run: cargo deb --variant cuda
```

- [ ] **Step 5: Edit the `deb-cuda` artifact upload path**

Find:

```yaml
      - uses: actions/upload-artifact@v4
        with:
          name: deb-cuda
          path: linux/target/debian/voice-input-cuda_*.deb
```

Replace the `path:` value:

```yaml
      - uses: actions/upload-artifact@v4
        with:
          name: deb-cuda
          path: target/debian/voice-input-cuda_*.deb
```

- [ ] **Step 6: Verify no `linux/` paths remain in the workflow**

```bash
grep -n 'linux' .github/workflows/release.yml
```

Expected: **empty output** (no matches). If anything matches, fix it before continuing.

- [ ] **Step 7: Verify the YAML still parses**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))" && echo "YAML OK"
```

Expected: `YAML OK`. If parsing fails, the indentation is wrong somewhere — re-check the four hunks above.

- [ ] **Step 8: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "$(cat <<'EOF'
ci: update release.yml paths after linux/ flatten

The previous commit moved Cargo.toml and target/ from linux/ to the
repo root, but release.yml still ran cargo deb with working-directory:
linux and uploaded artifacts from linux/target/debian/. Fix four hunks
to operate at the repo root.

Verified by yaml.safe_load; full end-to-end verification happens at
the next tag push.
EOF
)"
```

- [ ] **Step 9: Verify**

```bash
git log --oneline -3
```

Expected: top three commits are the CI fix, the flatten, and the artifact removal — in that order.

---

## Task 4: Switch LICENSE copyright to Desmond Chen

**Files:**
- Modify: `LICENSE` (one line)

- [ ] **Step 1: Inspect current LICENSE**

```bash
head -5 LICENSE
```

Expected:

```
MIT License

Copyright (c) 2025 yetone

Permission is hereby granted, free of charge, to any person obtaining a copy
```

- [ ] **Step 2: Edit the copyright line**

In `LICENSE`, replace:

```
Copyright (c) 2025 yetone
```

with:

```
Copyright (c) 2026 Desmond Chen
```

- [ ] **Step 3: Verify the diff is exactly one line**

```bash
git diff LICENSE
```

Expected output (whitespace may differ):

```
diff --git a/LICENSE b/LICENSE
@@ -1,4 +1,4 @@
 MIT License

-Copyright (c) 2025 yetone
+Copyright (c) 2026 Desmond Chen
```

If more than one line changed, undo (`git checkout -- LICENSE`) and retry.

- [ ] **Step 4: Commit**

```bash
git add LICENSE
git commit -m "$(cat <<'EOF'
docs(license): switch copyright to Desmond Chen

The Linux Rust crate was written from scratch — git log shows no
commits inherited from yetone's original repository. The LICENSE
file is the only place the original author's name remained. With
the repo formally reframed as a standalone Linux project, the
copyright line is updated to match. yetone is credited as
inspiration in the new README (next commit).
EOF
)"
```

---

## Task 5: Rewrite root README and CLAUDE.md

**Files:**
- Modify: `README.md` (6 surgical edits + new Credits section)
- Replace: `CLAUDE.md` (full rewrite — old content describes the macOS app)

- [ ] **Step 1: README edit 1 — strip "(Linux)" from the title**

In `README.md`, replace:

```markdown
# VoiceInput (Linux)
```

with:

```markdown
# VoiceInput
```

- [ ] **Step 2: README edit 2 — drop `cd linux` from the Build section**

In `README.md`, replace:

```markdown
```bash
cd linux
cargo build --release
```
```

with:

```markdown
```bash
cargo build --release
```
```

- [ ] **Step 3: README edit 3 — drop `cd linux` from the CUDA build snippet**

In `README.md`, replace:

```markdown
```bash
sudo apt install nvidia-cuda-toolkit
cd linux
cargo build --release --features cuda
```
```

with:

```markdown
```bash
sudo apt install nvidia-cuda-toolkit
cargo build --release --features cuda
```
```

- [ ] **Step 4: README edit 4 — fix the autostart install path (two occurrences)**

In `README.md`, replace:

```markdown
./linux/scripts/install-autostart.sh
```

with:

```markdown
./scripts/install-autostart.sh
```

Use `replace_all` (there are two occurrences in the Autostart section).

- [ ] **Step 5: README edit 5 — fix the fallback binary path explanation**

In `README.md`, find the paragraph beginning "The script picks `voice-input` from your `$PATH`":

```markdown
The script picks `voice-input` from your `$PATH`, or falls back to `linux/target/release/voice-input`. Pass an explicit path if you want a different binary:
```

Replace with:

```markdown
The script picks `voice-input` from your `$PATH`, or falls back to `target/release/voice-input`. Pass an explicit path if you want a different binary:
```

- [ ] **Step 6: README edit 6 — fix the Project layout reference**

In `README.md`, replace:

```markdown
See `../plans/2026-05-15-voice-input-linux/voice-input-linux.md` for the full design and `../plans/2026-05-15-voice-input-linux/implementations/` for per-phase implementation plans (Phase 0 through Phase 7).
```

with:

```markdown
See `plans/2026-05-15-voice-input-linux/voice-input-linux.md` for the full design and `plans/2026-05-15-voice-input-linux/implementations/` for per-phase implementation plans (Phase 0 through Phase 7).
```

- [ ] **Step 7: README edit 7 — append the Credits section at the very end**

Append to `README.md`:

```markdown

## Credits

Inspired by [yetone/voice-input-src](https://github.com/yetone/voice-input-src),
a Fn-key-driven voice input app for macOS. This project is a from-scratch
Linux reimplementation in Rust targeting Wayland compositors — none of the
original Swift code is included, but the UX shape (hold-to-talk capsule
overlay + LLM refinement) traces back to that work.
```

- [ ] **Step 8: Verify the README has no `linux/` path references left**

```bash
grep -n 'linux/\|cd linux\|(Linux)' README.md
```

Expected: empty output, OR matches that are inside narrative prose about the Linux compositor — but NOT in code fences or paths. If the only remaining matches are sentences like "Linux release" or "any Linux", that is fine.

Spot-check by reading the first 12 lines:

```bash
head -12 README.md
```

Expected: title is `# VoiceInput`, then the Wayland-native intro paragraph.

- [ ] **Step 9: Replace CLAUDE.md with the new content**

Overwrite `CLAUDE.md` (currently describes the macOS Swift app) with:

````markdown
# CLAUDE.md

This file guides Claude Code working in this repository.

VoiceInput is a Wayland-native hold-to-talk voice input app for Linux. Single Rust crate. The press-and-hold global shortcut is bound through the XDG GlobalShortcuts portal; audio is captured via cpal, sliced by a Silero ONNX VAD detector, transcribed by whisper.cpp (optionally on CUDA), optionally refined through an OpenAI-compatible chat completion, then pasted into the focused application via ydotool. The overlay UI is a GTK4 wlr-layer-shell capsule. Targets KDE Plasma 6, sway, and hyprland; **not supported on GNOME** (mutter lacks wlr-layer-shell).

## Build & Run

```bash
cargo build --release                         # CPU build
cargo build --release --features cuda         # NVIDIA GPU (5–15× faster transcription)

cargo run --release                           # full app: tray + listen + overlay
cargo run --release -- transcribe             # standalone segment-and-print mode
cargo test                                    # unit + integration tests
```

System packages (Debian/Ubuntu):

```
cmake clang libclang-dev libgtk-4-dev libgtk4-layer-shell-dev
libwayland-dev libxkbcommon-dev libasound2-dev pkg-config
```

CUDA build additionally requires `nvidia-cuda-toolkit`.

Runtime requirements:

- `ydotool` installed and the packaged user service running: `systemctl --user is-active ydotool.service` must report `active`.
- A Wayland compositor with `wlr-layer-shell` support (KDE Plasma 6, sway, or hyprland).
- A whisper.cpp model file under `~/.local/share/voice-input/models/` (default `ggml-small.bin`; the project also supports `ggml-large-v3-turbo.bin` for CUDA users).

## Architecture

```
KeyMonitor (XDG portal) ──▶  AppOrchestrator (app.rs)
                                  │
                                  ▼
                          AudioCapture (audio.rs, cpal)
                                  │
                                  ▼
                          VadSlicer (speech/vad.rs, Silero ONNX)
                                  │
                                  ▼
                          WhisperWorker (speech/worker.rs)
                                  │
                                  ▼   (optional)
                          LlmRefiner (refiner.rs)
                                  │
                                  ▼
                          TextInjector (injector.rs, ydotool)
                                  │
                                  ▼
                          focused application

TrayMenu (tray.rs, ksni)      ◀────  status / config
OverlayPanel (overlay/, GTK4) ◀────  waveform + state
```

Per-file responsibilities (`src/`):

- **`main.rs`** — entry point. Sets up logging, builds the tokio runtime, launches the GTK4 main loop on the OS main thread, spawns the async backend on a worker thread.
- **`lib.rs`** — library crate root. Re-exports the modules used by integration tests under `tests/`.
- **`app.rs`** — orchestrator. Owns the dictation lifecycle: subscribes to portal hotkey events, starts/stops audio capture, drives whisper transcription, optionally refines via LLM, paste-injects the final text.
- **`hotkey.rs`** — XDG portal `GlobalShortcuts` session wrapper. Binds the `toggle_recording` shortcut, exposes `activated`/`deactivated` event streams.
- **`audio.rs`** — cpal input stream management. Opens the default input device, posts raw samples + RMS to channels.
- **`speech/`** — `vad.rs` (Silero ONNX voice-activity slicer; stateful detector reset per dictation) and `worker.rs` (persistent whisper.cpp worker that loads the model once at startup and reuses `WhisperState` between dictations, saving ~1.5–2s startup cost per utterance).
- **`refiner.rs`** — optional OpenAI-compatible chat-completion client. Conservative system prompt: fix obvious ASR errors only, never rewrite.
- **`injector.rs`** — ydotool wrapper. wl-clipboard write + simulated Ctrl+V via the ydotoold daemon.
- **`tray.rs`** — ksni-based StatusNotifierItem. Menu: Enabled / Language / LLM Refinement → Settings / Quit. Switches icon between idle (`microphone`) and recording (`media-record-symbolic`).
- **`settings_window.rs`** — GTK4 dialog for editing LLM refiner settings (Base URL / API Key / Model) with Test and Save actions. Launched from the tray menu.
- **`overlay/`** — `window.rs` (GTK4 + wlr-layer-shell capsule centered at the bottom of the screen) and `waveform.rs` (5-bar animated waveform driven by audio RMS). Module entrypoint `mod.rs`.
- **`config.rs`** — TOML config at `~/.config/voice-input/config.toml`. Read at startup, written when tray menu options change.
- **`state.rs`** — shared mutable runtime state (current language, refiner enable/disable, etc.) accessed by tray, overlay, and orchestrator through `Arc<RwLock<…>>`.
- **`cli.rs`** — clap-derived CLI parsing for the subcommand modes (`transcribe`, `listen`, default).
- **`error.rs`** — shared `AppError` / `AppResult` types.

## Conventions When Editing

- **ydotool is the only paste mechanism.** Wayland does not allow synthesizing Ctrl+V from a non-privileged process via `wl-clipboard` alone. Any rewrite of `injector.rs` must preserve the dependency on a running `ydotool.service` (which owns `/dev/uinput`).
- **Do not package voice-input as a systemd `--user` service.** KDE's xdg-desktop-portal identifies the caller through the cgroup scope (`/.../app.slice/app-<app_id>-<PID>.scope`). A systemd `.service` cgroup does not match this pattern, and the portal silently refuses `CreateSession`. The supported launch path is the XDG autostart entry generated by `scripts/install-autostart.sh`. Troubleshooting details live in `README.md`.
- **CUDA arch must be explicit in CI.** whisper.cpp defaults `CMAKE_CUDA_ARCHITECTURES=native`. On a GPU-less CI runner that falls back to nvcc's hardcoded `sm_52`, producing a binary that aborts on any modern card. `.github/workflows/release.yml` sets `CMAKE_CUDA_ARCHITECTURES="75;80;86;89;90"` (Turing → Hopper). Do not remove that env var.
- **Persisted config keys are stable.** `~/.config/voice-input/config.toml` keys (`whisper_model_path`, `whisper_model_size`, `language_hint`, `llm_enabled`, `llm_api_base_url`, `llm_api_key`, `llm_model`, `llm_timeout_secs`) — do not rename without a migration.
- **Tray icons use freedesktop names**, not file paths: `microphone` for idle, `media-record-symbolic` for recording.

## Release Process

1. Bump `version = "..."` in `Cargo.toml` on `main`.
2. Add a corresponding entry to `CHANGELOG.md`.
3. Push `main`, then tag and push:
   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```
4. `.github/workflows/release.yml` builds CPU and CUDA `.deb` variants in parallel (CUDA takes ~30 min for 5 archs) and attaches both to the GitHub Release.
5. Verify both `voice-input_X.Y.Z_amd64.deb` and `voice-input-cuda_X.Y.Z_amd64.deb` are attached at `https://github.com/desmondc9/voice-input-src/releases/tag/vX.Y.Z`.
````

- [ ] **Step 10: Verify CLAUDE.md has no macOS terms left**

```bash
grep -i 'NSPanel\|Cmd+V\|TISSelect\|Swift\|macOS\|LSUIElement\|Fn key\|UserDefaults' CLAUDE.md
```

Expected: empty output.

- [ ] **Step 11: Commit**

```bash
git add README.md CLAUDE.md
git commit -m "$(cat <<'EOF'
docs: rewrite root README and CLAUDE.md for standalone Linux project

The README is the result of six surgical edits to the file that used
to be linux/README.md: strip "(Linux)" from the title, drop "cd linux"
from build snippets, fix script paths to drop the linux/ prefix, fix
the plans/ reference, and append a Credits section linking to
yetone/voice-input-src as inspiration.

CLAUDE.md is rewritten from scratch — the previous content described
the macOS Swift app (KeyMonitor, NSPanel, Cmd+V, TISSelectInputSource)
and bore no relationship to what this repo actually builds. The new
file documents the Rust crate: build commands, architecture diagram,
per-file responsibilities, the three load-bearing conventions
(ydotool dependency, no systemd --user service, explicit CUDA arch
in CI), and the release process.
EOF
)"
```

- [ ] **Step 12: Verify the final repository state**

```bash
git log --oneline -5
ls -la
cargo check 2>&1 | tail -3
```

Expected output of `git log`:

```
<hash> docs: rewrite root README and CLAUDE.md for standalone Linux project
<hash> docs(license): switch copyright to Desmond Chen
<hash> ci: update release.yml paths after linux/ flatten
<hash> refactor(repo): flatten linux/ to repo root
<hash> chore(repo): remove macOS-era artifacts
```

Expected `ls -la`: `Cargo.toml`, `Cargo.lock`, `src/`, `tests/`, `scripts/`, `README.md`, `CLAUDE.md`, `LICENSE`, `CHANGELOG.md`, `plans/`, `docs/`, `.github/`, `.gitignore` — and NO `linux/`, `dist/`, `README_CN.md`, `.gitmodules`.

Expected `cargo check`: `Finished` with no errors.

---

## Task 6: Final cross-cutting verification

This task makes no commits. It is a final readout.

- [ ] **Step 1: Repository structure check**

```bash
ls -la | awk '{print $NF}' | grep -v '^\.\.\?$' | sort
```

Expected (one entry per line, alphabetical):

```
.git
.github
.gitignore
CHANGELOG.md
CLAUDE.md
Cargo.lock
Cargo.toml
LICENSE
README.md
docs
input_test.txt    # working-tree only, gitignored
plans
scripts
src
target            # if you built; gitignored
tests
```

There must be NO `dist/`, NO `linux/`, NO `README_CN.md`, NO `.gitmodules`.

- [ ] **Step 2: Build from the new root succeeds**

```bash
cargo build --release
```

Expected: `Finished `release` profile`.

- [ ] **Step 3: Tests pass**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 4: README first paragraph reads correctly**

```bash
head -5 README.md
```

Expected: title `# VoiceInput` (no parenthetical), followed by the "Wayland-native voice input" paragraph.

- [ ] **Step 5: Credits section exists**

```bash
tail -10 README.md
```

Expected: a `## Credits` section that links to `https://github.com/yetone/voice-input-src`.

- [ ] **Step 6: LICENSE attribution is updated**

```bash
head -3 LICENSE
```

Expected:

```
MIT License

Copyright (c) 2026 Desmond Chen
```

- [ ] **Step 7: Workflow has no `linux/` paths**

```bash
grep 'linux' .github/workflows/release.yml
```

Expected: empty output.

- [ ] **Step 8: Verify the 5 expected commits are sitting on top of `main`**

```bash
git log --oneline -5
```

Expected (in order, top to bottom):

```
<hash> docs: rewrite root README and CLAUDE.md for standalone Linux project
<hash> docs(license): switch copyright to Desmond Chen
<hash> ci: update release.yml paths after linux/ flatten
<hash> refactor(repo): flatten linux/ to repo root
<hash> chore(repo): remove macOS-era artifacts
```

- [ ] **Step 9: Push to remote**

```bash
git push origin main
```

This publishes all 5 commits. The release workflow does NOT trigger from a push to `main` (it triggers from tag pushes only), so this is safe.

- [ ] **Step 10: Out-of-scope follow-ups (do NOT do as part of this plan)**

Note for after this plan lands — leave these for the user to do separately:

- `git remote remove upstream` — purely local git config hygiene; the `upstream` remote still points at `yetone/voice-input-src` which is conceptually wrong post-restructure
- Update the GitHub repo description on github.com → "Wayland-native voice input for Linux"
- Add GitHub topics: `linux`, `wayland`, `kde-plasma`, `whisper-cpp`, `voice-input`, `rust`
- Bump `actions/checkout@v4` → `v5` and `actions/upload-artifact@v4` → `v5` for the Node.js 24 transition (separate commit, before next tag)
- Eventual repo rename `voice-input-src` → `voice-input` — defer; breaks existing release-asset URLs in v0.1.0 and v0.1.1
