# Virtual Soundboard

Virtual Soundboard is a Windows desktop app for playing audio clips like a physical soundboard. It is free, and it is built to work with voice chat apps such as Discord and games.

The app mixes your microphone and soundboard clips into VB-Cable. When Discord or a game uses `CABLE Output (VB-Audio Virtual Cable)` as its input device, clips sound like they came from your microphone. The app can also monitor clips through your real speakers or headphones.

It is built with Tauri, Rust, TypeScript, and React.

## Features

- Import `.mp3` and `.wav` sound clips.
- Assign clips to soundboard pads.
- Add labels, global hotkeys, and per-pad volume.
- Trigger clips from inside the app or with global hotkeys.
- Route microphone plus clips into Discord or game chat through VB-Cable.
- Hear clips through your selected monitor output without hearing your own mic.
- Toggle mic-only DSP effects.
- Save clips, layout, device choices, hotkeys, playback config, and effects locally.

## Requirements

- Windows
- Node.js
- Rust
- VB-Cable for voice chat routing

Download VB-Cable from VB-Audio:

https://vb-audio.com/Cable/

## Run in Development

Install dependencies:

```powershell
npm install
```

Start the dev app:

```powershell
npm run tauri dev
```

## Build the App

Create a release build:

```powershell
npm run tauri build
```

The release `.exe` will be in:

```text
src-tauri\target\release\
```

If Tauri creates an installer, it will be in:

```text
src-tauri\target\release\bundle\
```

## Basic Setup

1. Install VB-Cable.
2. Start the app.
3. On the Main tab, choose your microphone input.
4. Choose your clip monitor output, such as speakers or headphones.
5. Start the audio engine.
6. In Discord or your game, set microphone/input to:

```text
CABLE Output (VB-Audio Virtual Cable)
```

The app sends audio to:

```text
Mic + clips -> VB-Cable -> Discord/game
Clips only  -> speakers/headphones
```

## Mic Effects

Mic effects are applied only to live microphone audio before it enters the voice-chat mix. They do not affect soundboard clips.

Current proof-of-concept effects:

- Noise Gate: reduces low-level background noise.
- High-Pass Filter: removes low rumble.
- Low-Pass Filter: removes high-frequency harshness.
- Soft Saturation: adds gentle drive/compression-style color.

The Rust audio callback reads effect settings through atomics instead of locking a normal mutex. This keeps the real-time audio path cheap and makes it easier to add higher-quality DSP later.

## Global Hotkeys

Hotkeys are configured per soundboard cell. They are registered with the operating system only while the audio engine is running.

- Starting the audio engine registers the current hotkeys.
- Editing cell hotkeys while the engine is running re-registers them.
- Stopping the audio engine unregisters all hotkeys.

## Local Persistence

Saved app data lives in Tauri's app data directory, not in browser `localStorage`.

The backend creates:

```text
app_data_dir\clips\
app_data_dir\soundboard-layout.json
```

Imported clips are copied into `clips`. The layout JSON stores grid size, cell assignments, labels, hotkeys, volume, selected devices, playback settings, and mic effects.

## Project Map

Frontend:

- `src/App.tsx`: main app state, Tauri command calls, persistence flow.
- `src/defaults.ts`: shared frontend defaults for grid size and mic effects.
- `src/types.ts`: frontend data types.
- `src/soundboardState.ts`: soundboard grid creation and volume clamping.
- `src/components/MainTab.tsx`: device selection, engine controls, mic test.
- `src/components/SoundboardTab.tsx`: pad grid, cell editing, hotkey capture.
- `src/components/AudioClipManagerTab.tsx`: clip import/delete UI.
- `src/components/ConfigTab.tsx`: playback settings and mic effects controls.

Backend:

- `src-tauri/src/lib.rs`: Tauri setup, app state, commands, global shortcut plugin.
- `src-tauri/src/soundboard.rs`: soundboard state, clip storage, device discovery, engine lifecycle.
- `src-tauri/src/audio.rs`: CPAL input/output streams, mic passthrough, clip mixing, monitor playback.
- `src-tauri/src/effects.rs`: real-time-safe mic effects config and DSP processing.
- `src-tauri/src/clip.rs`: MP3/WAV decoding and clip sample playback.

## Voice Chat Settings

In Discord or other voice chat apps, soundboard clips may get filtered out unless you change noise settings.

Recommended:

- Turn off echo cancellation.
- Turn off noise suppression.
- Lower the noise gate as far as possible.
- Disable automatic input sensitivity if it cuts clips off.

## Troubleshooting

### VB-Cable says it is not installed

Make sure Windows shows an output device named something like:

```text
CABLE Input (VB-Audio Virtual Cable)
```

Then restart the app or press **Scan** on the Main tab.

### I do not hear my clips

Check these:

- The audio engine is running.
- The pad has a clip assigned.
- The Config tab has **Hear my audio clips** enabled.
- The clip monitor output is your real speakers/headphones, not VB-Cable.
- The per-pad volume is not very low.

### I hear clips, but nobody hears my mic

Check that the Main tab input device is your actual microphone.

Also make sure Discord/game input is `CABLE Output`, not your physical microphone.

### Effects are not changing clips

That is expected. Effects are mic-only. Soundboard clips remain clean.

### AirPods

Currently, Apple AirPods do not work with this application.

### My saved clips/layout are not included when I share the `.exe`

Correct. Saved clips, hotkeys, layout, device choices, and effects are stored locally in your app data folder. Friends who run the app will start with their own fresh setup.
