# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Layout

This repo is intentionally minimal — it is a **reproducibility wrapper** around a single-prompt build. The product itself (the macOS `VoiceInput.app`) lives in the `dist/` git submodule (https://github.com/yetone/voice-input-dist).

- `README.md` / `README_CN.md` — the **full Claude prompt** used to generate the app from scratch. These prompts are the spec; the source in `dist/` is the agent's output.
- `dist/` — git submodule containing the actual Swift Package, `Makefile`, `Info.plist`, and `Sources/VoiceInput/`. All build commands run there.

When cloning fresh, initialize the submodule:

```bash
git submodule update --init --recursive
```

## Build & Run

All commands must be run from `dist/`:

```bash
cd dist
make build    # swift build -c release, assemble .app bundle, ad-hoc codesign
make run      # build then `open VoiceInput.app`
make install  # copy bundle to /Applications
make clean    # swift package clean + rm -rf VoiceInput.app
```

Requirements: macOS 14+ (Sonoma) and Xcode Command Line Tools. There is no test target in `Package.swift`.

Runtime logs (LLM refiner request/response) are written to `~/Library/Logs/VoiceInput.log`.

## Architecture

Single-target Swift Package, all sources in `dist/Sources/VoiceInput/`. The app is an `LSUIElement` (menu-bar only, no Dock icon) wired together in `AppDelegate`:

```
KeyMonitor  ──fn down/up──▶  AppDelegate  ──▶  SpeechEngine  ──partial/final text──▶  OverlayPanel
                                  │                                                       │
                                  │                                          audio RMS ◀──┘
                                  ▼
                            LLMRefiner (optional)
                                  ▼
                            TextInjector  ──Cmd+V──▶ focused app
```

Per-file responsibilities:

- **`main.swift`** — sets `NSApplication` activation policy to `.accessory` and runs the app.
- **`AppDelegate.swift`** — orchestrator. Owns the status item, menus (Enable, Language, LLM Refinement → Settings, Quit), wires `KeyMonitor` callbacks to recording lifecycle, and decides whether to refine via LLM before injecting.
- **`KeyMonitor.swift`** — `CGEvent` tap on `.flagsChanged` watching `.maskSecondaryFn`. Returns `nil` from the callback to **suppress the Fn event** (otherwise the OS would open the emoji picker). Requires Accessibility permission; `start()` returns `false` if missing.
- **`SpeechEngine.swift`** — `SFSpeechRecognizer` + `AVAudioEngine` streaming recognition. Computes RMS from the input buffer per tap, normalizes to 0–1, fires `onAudioLevel` for the waveform. Default locale `zh-CN`; changing `locale` re-creates the recognizer.
- **`OverlayPanel.swift`** — borderless `NSPanel` (`.nonactivatingPanel`, `.canJoinAllSpaces`, `.fullScreenAuxiliary`) centered at the bottom of the screen, `NSVisualEffectView` (`.hudWindow`) with capsule corner radius. Contains a `WaveformView` (5 weighted bars, attack 0.4 / release 0.15 smoothing) driven by real audio RMS, and an elastic-width label (160–560 px). Spring entry (0.35s), width transition (0.25s), exit scale (0.22s).
- **`LLMRefiner.swift`** — singleton (`.shared`). OpenAI-compatible `/chat/completions` client with a **deliberately conservative** system prompt: fix only obvious recognition errors (e.g. `配森→Python`, `杰森→JSON`), never rewrite or polish. Settings persisted in `UserDefaults` (`llmEnabled`, `llmAPIBaseURL`, `llmAPIKey`, `llmModel`). `refine(force:)` bypasses the enabled/configured guard so Settings → Test works.
- **`TextInjector.swift`** — clipboard + simulated Cmd+V. Critical detail: if the active input source is **not ASCII-capable** (i.e. a CJK IME), it temporarily selects an ASCII source (preferring `com.apple.keylayout.ABC` / `.US`) via `TISSelectInputSource`, posts the Cmd+V keystroke, then restores the original input source after 0.3s and the original clipboard contents after 0.5s. Without this swap, CJK IMEs intercept Cmd+V.
- **`SettingsWindow.swift`** — Settings panel for LLM (Base URL / API Key / Model + Test + Save).

## Conventions When Editing

- `Info.plist` must keep `LSUIElement=true`, `NSMicrophoneUsageDescription`, and `NSSpeechRecognitionUsageDescription` — without the usage strings the app crashes on first permission request.
- The Fn-key handler in `KeyMonitor.handle` **must return `nil`** for the press/release transitions; returning the event re-introduces the emoji picker bug.
- `TextInjector` ordering — switch input source → 50µs settle → post Cmd+V → restore source at +0.3s → restore clipboard at +0.5s. Changing these delays tends to break either the paste or the restore.
- `LLMRefiner`'s system prompt is intentionally conservative. If you tune it, preserve the "return as-is when in doubt" guarantee — users have explicitly asked for no rewriting/polishing.
- Persisted `UserDefaults` keys in use: `selectedLocaleCode`, `llmEnabled`, `llmAPIBaseURL`, `llmAPIKey`, `llmModel`. Don't rename these without a migration.

## Reproducibility Note

The README prompt is the contract: the source in `dist/` is supposed to be regenerable from it. If you change behavior here, consider whether the README prompt should be updated to match — the project advertises that running the prompt reproduces the artifact.
