# Plan: Adding Linux Support to VoiceInput

> **File-location note**: plan-mode only permits writes to `~/.claude/plans/`. After approval, copy this file to the project's `./plans/voice-input-linux.md` (you'll need to `mkdir -p plans` first — the directory doesn't exist yet).

## Context

`voice-input-src` is a macOS-only menu-bar voice input app built on Apple Speech (`SFSpeechRecognizer`), AppKit (`NSPanel`, `NSStatusItem`, `NSVisualEffectView`), Quartz (`CGEvent` tap on `.maskSecondaryFn`), and Carbon (`TIS*` input source APIs). Every subsystem hits an Apple-proprietary framework — there is essentially nothing to share with Linux. Adding Linux support is therefore a **parallel rewrite**, not a refactor.

Goal: ship a Linux-native binary that reproduces the macOS feature set (hold-to-record hotkey → streaming transcription → optional conservative LLM refinement → paste into focused app → animated capsule overlay with live RMS waveform) on **Wayland (KDE Plasma 6, sway, hyprland)** using **Rust** and **local whisper.cpp**. GNOME is explicitly out of scope (mutter lacks `wlr-layer-shell`).

## Strategic decisions (already chosen)

| Axis | Choice | Why |
|---|---|---|
| Speech engine | whisper.cpp (local) | Offline, private, multilingual incl. zh-CN, MIT |
| Display server | Wayland-first (Plasma 6 / sway / hyprland) | Where the user actually lives |
| Language | Rust | Mature crates for every subsystem, single static binary |
| Scope | Full feature parity with macOS | Capsule overlay, RMS waveform, LLM refiner, language menu |

## Repository layout

The project's reproducibility-wrapper convention (README prompt → `dist/` submodule) is macOS-specific. Mirror it for Linux:

```
voice-input-src/
├── README.md             # existing macOS prompt
├── README_LINUX.md       # NEW: parallel `claude -p "..."` prompt for the Linux build
├── dist/                 # existing macOS submodule
└── linux/                # NEW: Linux source (could later be promoted to its own submodule)
    ├── Cargo.toml
    ├── README.md         # Linux build/install/troubleshooting
    ├── packaging/        # AppImage + Flatpak manifests
    └── src/
        ├── main.rs       # entry: tokio runtime, GTK init, wires modules
        ├── app.rs        # central state machine (AppDelegate equivalent)
        ├── hotkey.rs     # ashpd GlobalShortcuts + compositor-binding IPC fallback
        ├── audio.rs      # cpal capture + RMS computation
        ├── speech.rs     # whisper-cpp-plus streaming wrapper
        ├── overlay/      # GTK4 + gtk4-layer-shell capsule window
        │   ├── mod.rs
        │   └── waveform.rs  # 5-bar DrawingArea, attack/release smoothing + jitter
        ├── injector.rs   # wl-clipboard-rs + ashpd RemoteDesktop (ydotool fallback)
        ├── refiner.rs    # OpenAI-compatible HTTP refiner (direct port)
        ├── tray.rs       # ksni status notifier
        ├── settings.rs   # GTK4 dialog mirroring SettingsWindow.swift
        └── config.rs     # serde + toml at ~/.config/voice-input/config.toml
```

## Module port map

| macOS source (`dist/Sources/VoiceInput/`) | Linux module (`linux/src/`) | Crate(s) | Notes |
|---|---|---|---|
| `main.swift` | `main.rs` | `tokio`, `gtk4` | Init GTK, register portal, spawn tray, run main loop |
| `AppDelegate.swift` | `app.rs` | `tokio::sync` channels | Orchestrator state machine (Idle / Listening / Refining / Injecting). Same callbacks/sequencing as the Swift version. |
| `KeyMonitor.swift` (CGEvent Fn tap) | `hotkey.rs` | `ashpd` (portal `GlobalShortcuts`) | Use `Session::create_shortcuts` + `receive_activated()` / `receive_deactivated()` for true press/release. **Fn is unavailable on Linux** — default is user-chosen via the portal binding dialog. Document a sway/hyprland fallback that binds in the compositor config and pokes the app over a unix socket. |
| `SpeechEngine.swift` (SFSpeechRecognizer + AVAudioEngine + RMS) | `audio.rs` + `speech.rs` | `cpal`, `whisper-cpp-plus` (preferred) or `whisper-rs` | Audio capture and RMS in `audio.rs` (one tap, two consumers: whisper feed + RMS callback). `speech.rs` wraps `WhisperStream` with sliding window. Model path resolved from config or first-run download. Expect 600 ms–1.2 s partial latency on `whisper-small` CPU — UI must accept that partials *rewrite*. |
| `OverlayPanel.swift` + `WaveformView` | `overlay/mod.rs` + `overlay/waveform.rs` | `gtk4`, `gtk4-layer-shell`, `cairo-rs` | `Window` → set `layer = Overlay`, `anchor = Bottom`, `margin = 56`, `keyboard_interactivity = None`, `exclusive_zone = -1`. Capsule via CSS `border-radius` + `background: alpha(...)` on a `GtkBox`. Waveform is a `DrawingArea` painted by hand to keep the same `[0.5, 0.8, 1.0, 0.75, 0.55]` weights, attack 0.4 / release 0.15 smoothing, ±4% jitter. Width animation via `gtk::Adjustment` + `glib::timeout_add_local` (matches the Swift 0.25s ease). |
| `LLMRefiner.swift` | `refiner.rs` | `reqwest`, `serde_json`, `tokio` | Pure HTTP — direct port. Keep the conservative system prompt **verbatim** (it's part of the product contract). `force` flag for Settings → Test. |
| `TextInjector.swift` (clipboard + Cmd+V + IME swap) | `injector.rs` | `wl-clipboard-rs`, `ashpd` (`RemoteDesktop`), `tokio::process` (ydotool fallback) | 1) snapshot clipboard → 2) write transcription → 3) emit Ctrl+V via portal RemoteDesktop session (preferred), shell out to `ydotool key ctrl+v` if portal unavailable → 4) restore original clipboard after ~500 ms. **IME swap is unnecessary on Linux** — fcitx5/ibus don't intercept Ctrl+V the way macOS CJK IMEs intercept Cmd+V. Drop the swap logic. |
| `SettingsWindow.swift` | `settings.rs` | `gtk4` | Three `Entry` widgets (Base URL / API Key / Model) + Test + Save buttons. Same persistence. |
| `setupStatusBar()` in AppDelegate | `tray.rs` | `ksni` | StatusNotifierItem with Enabled / Language submenu / LLM Refinement (Enable + Settings…) / Quit. Submenus map cleanly. |
| `UserDefaults` keys | `config.rs` | `serde`, `toml`, `directories` | TOML at `~/.config/voice-input/config.toml`: `selected_locale_code`, `llm_enabled`, `llm_api_base_url`, `llm_api_key`, `llm_model`, `hotkey_id`, `whisper_model_path`. |
| `NSLog` + `~/Library/Logs/VoiceInput.log` | (cross-cutting) | `tracing`, `tracing-subscriber` | Write to `~/.local/state/voice-input/voice-input.log` (XDG state dir). |

## Wayland gotchas & mitigations

1. **Portal GlobalShortcuts requires interactive binding on first run.** You cannot ship "hold Fn" as a default — the portal shows a system dialog and the user picks a chord. Plan a one-time onboarding screen explaining this. For sway/hyprland users who prefer compositor-native binding, document `bindsym --no-repeat $mod+space +/-` style configs that signal a unix socket the app listens on.
2. **virtual-keyboard protocol does NOT work on KDE Plasma 6.** This rules out `wtype` and `enigo`'s Wayland backend as a primary path. Use **portal RemoteDesktop (`ashpd::desktop::remote_desktop`)** as primary keystroke injector; fall back to **ydotool** (requires `ydotoold` running with `/dev/uinput` access — document the udev/group setup).
3. **Layer-shell isn't on GNOME mutter.** Don't try to support GNOME in scope. Detect mutter at startup and surface a clear error.
4. **Clipboard contents die with the offering client.** Keep the process alive through the entire paste sequence (we already do — sequential async, not detached). Add a deliberate ~500 ms delay before restoring original contents to make sure the paste latched.
5. **Tray hosting on sway/hyprland needs an SNI host.** waybar (or ironbar/Riftbar) must be running with the `tray` module. Document this; on Plasma 6 the host is built-in.
6. **Whisper partials churn.** Unlike SFSpeechRecognizer, whisper streams *rewrite* earlier text as context grows. Mirror this honestly in the overlay — keep the existing 0.25s width-transition so the rewrites feel like elegant settling rather than jitter.
7. **Microphone permissions.** Linux doesn't gate this like macOS does, but PipeWire's portal (`org.freedesktop.portal.Camera`-style for audio) is emerging. For now, document that the user's regular audio stack handles it; no explicit permission code needed.

## Hotkey default

Fn is firmware-handled on most Linux laptops and not visible to userspace. Replace with a **user-chosen chord via the portal dialog**. Suggested guidance for the binding dialog: **Right Ctrl** (single key, easy to hold, rarely bound elsewhere) or **Super+Space**. Persist the portal-issued shortcut handle in config so reconnection is silent on subsequent runs.

## Build & distribution

- Toolchain: stable Rust 1.83+, `cmake`, `gcc`/`clang`, `libgtk-4-dev`, `libgtk4-layer-shell-dev`, `pkg-config`, plus `pipewire-jack`/`libpipewire-0.3-dev` (cpal pulls these on PipeWire systems).
- Build: `cargo build --release` — single static-ish binary in `linux/target/release/voice-input`.
- Whisper model: on first run, prompt to download `ggml-small.bin` (~466 MB) from Hugging Face into `~/.local/share/voice-input/models/`. Allow `medium` for higher accuracy via the Settings UI.
- Packaging: **AppImage first** (linuxdeploy-plugin-gtk) — easy distribution. **Flatpak second** when the portal flows have been hardened (Flatpak forces all portals on, which actually tightens the design).
- Autostart: install `~/.config/autostart/voice-input.desktop` from the Settings UI.

## Phased build sequence

1. **Phase 0 — scaffold (1–2 days):** `Cargo.toml` + workspace, GTK4 hello-world, `ksni` "Quit" tray, `config.rs` round-trip. Validate the toolchain on the target compositors.
2. **Phase 1 — audio + speech (2–3 days):** `cpal` capture + RMS, integrate `whisper-cpp-plus` with `whisper-small`, print partials/finals to stdout. No UI yet.
3. **Phase 2 — hotkey + paste loop (2–3 days):** `ashpd` GlobalShortcuts press/release wired to record-start/record-stop. `wl-clipboard-rs` save/restore. Portal RemoteDesktop Ctrl+V (with ydotool fallback). End-to-end: hold key, speak, release, see text pasted into another window.
4. **Phase 3 — overlay (3–4 days):** `gtk4-layer-shell` capsule, animated width transition, custom-drawn 5-bar waveform with the exact weights/smoothing constants from `OverlayPanel.swift`. Refining state.
5. **Phase 4 — LLM refiner (1 day):** direct port of `LLMRefiner.swift`; copy the system prompt verbatim.
6. **Phase 5 — Settings + tray menus (1–2 days):** GTK Settings dialog, Language submenu (re-binds whisper model/locale), Enable toggle, LLM Refinement submenu.
7. **Phase 6 — packaging (2 days):** AppImage build, first-run model download UX, README with compositor matrix + troubleshooting.

Total rough estimate: ~2 weeks of focused work.

## Critical files referenced from the macOS source

Port behavior 1:1 from these — they encode product decisions worth preserving:
- `dist/Sources/VoiceInput/LLMRefiner.swift:46-63` — system prompt (copy verbatim)
- `dist/Sources/VoiceInput/SpeechEngine.swift:97-103` — RMS → 0–1 normalization (`(dB + 50) / 40`, clamp)
- `dist/Sources/VoiceInput/OverlayPanel.swift:181-217` — waveform weights, attack/release, jitter
- `dist/Sources/VoiceInput/OverlayPanel.swift:97-135` — entry/width/exit animation timings (0.35 / 0.25 / 0.22)
- `dist/Sources/VoiceInput/AppDelegate.swift:111-171` — finish-transcription flow including the refining-then-inject sequencing

## Verification (end-to-end)

Test on **KDE Plasma 6**, **sway**, and **hyprland**:

1. Launch binary → tray icon appears (Plasma) or appears in waybar (sway/hyprland).
2. First run: portal prompts to bind a global shortcut → bind to Right Ctrl.
3. Open a text editor (Kate, gedit, alacritty + nvim). Focus the input field.
4. Hold Right Ctrl → capsule fades in centered at bottom, waveform reacts to speech RMS in real time.
5. Speak `"Python 和 JSON"` — partials stream in (expect rewriting); waveform tracks audio level.
6. Release Right Ctrl → if LLM refiner enabled, "Refining…" state shows; otherwise paste happens immediately.
7. Confirm: text lands in the focused editor; original clipboard contents (set a known string beforehand) restore after ~500 ms.
8. Change language to 日本語 in the tray submenu → repeat with Japanese phrase.
9. Open Settings → enter API base URL / API key / model → Test → see "OK: …" — confirms the refiner round-trip works.
10. Toggle "Enabled" off → hotkey no longer captures.
11. Quit from tray → process exits cleanly, no orphan GTK windows.

Non-functional checks:
- `whisper-small` partial latency under 1.2 s on the target machine.
- Capsule animation stays at 60 fps (use `GTK_DEBUG=interactive`).
- Memory after 50 record-cycles flat (no whisper context leak).

## Open risks / non-goals

- **GNOME is not supported.** Documented limitation.
- **Portal GlobalShortcuts on sway is the least-mature portal backend.** Compositor-binding fallback is the safety net.
- **whisper-cpp-plus is a small crate** — review its source before depending on it; fall back to hand-rolled sliding window over `whisper-rs` if it doesn't pass smell test.
- **Code sharing with macOS Swift: zero.** This is a parallel implementation. The only shared artifact is the LLM system prompt (copy verbatim) and the product spec (the README prompt).
- The reproducibility-prompt model assumes a single agent run produces the artifact — a Rust+GTK4+whisper.cpp project may exceed that one-shot scope. Acceptable: the Linux README prompt can be the design doc rather than a guaranteed one-shot reproducer.
