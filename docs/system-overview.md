# System Overview

This project has two main halves:

- A React/TypeScript frontend in `src/`.
- A Rust/Tauri backend in `src-tauri/src/`.

The frontend owns the user interface and app state that users edit directly: selected tab, selected devices, soundboard grid, clip assignments, hotkeys, volumes, playback options, and mic effect settings.

The backend owns operating-system work: audio devices, audio streams, file persistence, clip decoding, global hotkeys, and real-time DSP.

## Runtime Shape

At runtime, Tauri embeds the React UI in a desktop window. React calls Rust through Tauri commands:

```text
React component
-> App.tsx handler
-> invoke("command_name", payload)
-> #[tauri::command] Rust function
-> Soundboard / AudioEngine / file system / audio device
```

The most important backend state is:

```rust
struct AppState {
    soundboard: Mutex<Soundboard>,
    layout_path: PathBuf,
}
```

`AppState` is registered once during Tauri setup in `src-tauri/src/lib.rs`. Each command receives access to that shared state through `tauri::State<AppState>`.

The `Mutex<Soundboard>` protects shared mutable backend state. If one command is mutating the soundboard, another command must wait before it can access it.

## Main Audio Goal

The app makes this routing possible:

```text
physical microphone
-> mic effects
-> mic passthrough buffer
-> VB-Cable output
-> Discord/game input
```

Soundboard clips are mixed into the same VB-Cable output:

```text
soundboard clip
-> decoded sample buffer
-> clip player
-> VB-Cable output mix
```

Optionally, clips are also sent to a second monitor output:

```text
soundboard clip
-> monitor output
-> user's headphones/speakers
```

The monitor output intentionally receives clips only, not the user's microphone. That avoids hearing your own voice delayed back at you.

## Core Files

Frontend:

- `src/App.tsx`: top-level state and Tauri command orchestration.
- `src/components/MainTab.tsx`: device selection, engine controls, mic test.
- `src/components/SoundboardTab.tsx`: pad grid, cell editing, hotkey capture.
- `src/components/AudioClipManagerTab.tsx`: clip import/delete UI.
- `src/components/ConfigTab.tsx`: playback and mic effect settings.
- `src/defaults.ts`: frontend default values.
- `src/types.ts`: TypeScript data contracts.
- `src/soundboardState.ts`: grid creation and volume clamping.

Backend:

- `src-tauri/src/lib.rs`: Tauri setup, commands, app state, global shortcut plugin.
- `src-tauri/src/soundboard.rs`: high-level soundboard model and audio engine lifecycle.
- `src-tauri/src/audio.rs`: CPAL stream construction and sample mixing.
- `src-tauri/src/clip.rs`: MP3/WAV decoding and clip playback.
- `src-tauri/src/effects/mod.rs`: mic effects config and effect chain.
- `src-tauri/src/effects/simple.rs`: basic DSP effects.

## Common User Action Flow

Starting the audio engine:

```text
MainTab button
-> App.startAudioEngine
-> invoke("start_audio_engine")
-> lib.rs start_audio_engine command
-> Soundboard.start_audio_engine
-> AudioEngine::start
-> CPAL input/output streams begin
-> App registers global hotkeys
```

Playing a pad:

```text
SoundboardTab pad click
-> App.playCell
-> invoke("play_clip")
-> lib.rs play_clip command
-> Soundboard.play_clip
-> AudioEngine.play_clip
-> AudioCommand::PlayClip sent to output stream
-> output callback mixes clip samples
```

Importing a clip:

```text
AudioClipManagerTab upload button
-> Tauri file dialog
-> App.uploadClips
-> invoke("import_clip")
-> Soundboard.import_clip
-> copy file into app data clips directory
-> decode clip into memory
-> refresh clip list
```

Changing mic effects:

```text
ConfigTab control
-> App.changeMicEffectsConfig
-> invoke("set_mic_effects_config")
-> Soundboard.set_mic_effects_config
-> SharedMicEffects atomics update
-> input audio callback observes new values
```

## Important Design Choices

- React state is the source of truth for editable UI state.
- Rust state is the source of truth for audio engine runtime state.
- Layout and settings are persisted as JSON by Rust, not by browser local storage.
- Clips are copied into the app data directory so they survive restarts.
- Global hotkeys are only registered while the audio engine is running.
- Mic effects affect microphone audio only. They do not process soundboard clips.
- The real-time audio callback avoids normal mutex locking for mic effect settings.
