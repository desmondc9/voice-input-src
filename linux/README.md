# VoiceInput (Linux)

Wayland-native voice input for KDE Plasma 6, sway, and hyprland. Hold a configured key, speak, release — the transcript is pasted into the focused application.

> Status: **Phase 1** — CLI transcription pipeline. Tray still works (default invocation). No hotkey or paste injection yet (Phase 2). See `../implementation/` for the phased build plan.

## Build

Requires Rust 1.83+, `cmake`, `libclang`, and `cc`/`gcc`. On first build, whisper.cpp compiles from source (≈30–60 s).

```bash
cd linux
cargo build --release
```

System packages (Debian/Ubuntu): `sudo apt install cmake clang libclang-dev libasound2-dev`.

## Download a whisper model

Phase 1 expects a `ggml-*.bin` whisper model on disk. Default path: `~/.local/share/voice-input/models/ggml-small.bin`. To download:

```bash
mkdir -p ~/.local/share/voice-input/models
curl -L -o ~/.local/share/voice-input/models/ggml-small.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin
```

Other sizes: `tiny` (75 MB), `base` (142 MB), `small` (466 MB, default), `medium` (1.5 GB). Match the size to the `whisper_model_size` value in `~/.config/voice-input/config.toml`.

Override the path with `VOICE_INPUT_MODEL_PATH=/some/where/model.bin` or by setting `whisper_model_path` in the config file.

## Run

### Tray mode (Phase 0 behavior)

```bash
RUST_LOG=info cargo run
```

A tray icon appears in your system tray (KDE Plasma) or waybar (sway / hyprland — needs the `tray` module).

### Transcribe mode (Phase 1)

```bash
RUST_LOG=info cargo run -- transcribe
```

Reads from the default microphone, slices speech on natural pauses (≥300 ms silence), and prints each transcribed segment. Press Ctrl+C to stop.

Example output:
```text
[segment 1] 你好世界
[segment 2] this is a test
```

## Compositor support

- **KDE Plasma 6**: target compositor, built-in StatusNotifierItem host.
- **sway**: requires waybar with `tray` module.
- **hyprland**: requires waybar / ironbar / Riftbar with `tray` module.
- **GNOME**: **not supported.** Mutter lacks `wlr-layer-shell` (needed in Phase 3).

## Config

`~/.config/voice-input/config.toml` — created on first run. Edit and restart to change. Notable keys:

- `language_hint = "zh"` — passed to whisper as a hint (`"en"`, `"ja"`, etc., or empty for auto-detect)
- `whisper_model_size = "small"` — determines the default model path

## Project layout

See `../plans/voice-input-linux.md` for the full design and `../implementation/` for per-phase implementation plans.
