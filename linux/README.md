# VoiceInput (Linux)

Wayland-native voice input for KDE Plasma 6, sway, and hyprland. Hold a configured key, speak, release — the transcript is pasted into the focused application.

> Status: **Phase 2** — hotkey-driven dictation works end-to-end via the XDG portal + ydotool. Tray mode (default invocation) and transcribe CLI mode (Phase 1) both still work.

## Build

Requires Rust 1.83+, `cmake`, `libclang`, and `cc`/`gcc`. On first build, whisper.cpp compiles from source (≈30–60 s).

```bash
cd linux
cargo build --release
```

System packages (Debian/Ubuntu): `sudo apt install cmake clang libclang-dev libasound2-dev`.

## Download a whisper model

```bash
mkdir -p ~/.local/share/voice-input/models
curl -L -o ~/.local/share/voice-input/models/ggml-small.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin
```

Sizes: `tiny` (75 MB), `base` (142 MB), `small` (466 MB, default), `medium` (1.5 GB). Override the path with `VOICE_INPUT_MODEL_PATH=/path/to/model.bin` or by setting `whisper_model_path` in `~/.config/voice-input/config.toml`.

## Install ydotool (for `listen` mode only)

The listen mode pastes text by invoking `ydotool` to simulate Ctrl+V. This requires the `ydotoold` daemon to be running with `/dev/uinput` access. Bootstrap with the included script:

```bash
./scripts/install-ydotool.sh
```

The script installs the `ydotool` package, adds a udev rule, joins you to the `input` group, and enables a systemd user service for `ydotoold`. **Log out and log back in afterward** for group membership to take effect.

## Run

### Tray mode (Phase 0 behavior)

```bash
RUST_LOG=info cargo run
```

A tray icon appears in your system tray (KDE Plasma) or waybar (sway / hyprland — needs the `tray` module).

### Transcribe mode (Phase 1)

```bash
RUST_LOG=info cargo run --release -- transcribe
```

> Use `--release`. Debug-profile whisper inference is 5–10× slower; you'll get poor results without it.

Reads from the default microphone, slices speech on natural pauses (≥300 ms silence), and prints each transcribed segment. Press Ctrl+C to stop.

Example output:
```text
[segment 1] 你好世界
[segment 2] this is a test
```

### Listen mode (Phase 2)

```bash
RUST_LOG=info cargo run --release -- listen
```

First run: the XDG portal prompts you to bind a global shortcut. Recommended: **Right Ctrl** (single key, easy to hold, rarely conflicts).

Then: focus any text input, hold the configured key, speak, release. The transcript is pasted into the focused field. Press Ctrl+C in the terminal to stop the daemon.

Requires the steps in "Install ydotool" above.

## Compositor support

- **KDE Plasma 6**: target compositor, built-in StatusNotifierItem host. Portal `GlobalShortcuts` works out of the box.
- **sway**: requires waybar with `tray` module. Portal support via `xdg-desktop-portal-wlr` (less mature than KDE; document a compositor-binding fallback if portal proves unreliable).
- **hyprland**: requires waybar / ironbar / Riftbar with `tray` module. Portal support via `xdg-desktop-portal-hyprland`.
- **GNOME**: **not supported.** Mutter lacks `wlr-layer-shell` (needed in Phase 3).

## Config

`~/.config/voice-input/config.toml` — created on first run. Edit and restart to change. Notable keys:

- `language_hint = "zh"` — passed to whisper as a hint (`"en"`, `"ja"`, etc., or empty for auto-detect)
- `whisper_model_size = "small"` — determines the default model path
- `shortcut_handle` — reserved for explicit portal-handle persistence; currently unused (the portal remembers the binding internally)

## Project layout

See `../plans/voice-input-linux.md` for the full design and `../implementation/` for per-phase implementation plans.
