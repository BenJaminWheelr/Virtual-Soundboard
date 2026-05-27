# Virtual Soundboard

Virtual Soundboard is a Windows desktop app for playing audio clips like a physical soundboard. It is built to work with voice chat apps such as Discord and games by mixing microphone audio and soundboard clips into VB-Cable.

The app is built with Tauri, Rust, TypeScript, and React.

## Quick Start

Install dependencies:

```powershell
npm install
```

Run the app in development:

```powershell
npm run tauri dev
```

Create a release build:

```powershell
npm run tauri build
```

## Requirements

- Windows
- Node.js
- Rust
- VB-Cable for voice chat routing

Download VB-Cable from VB-Audio:

https://vb-audio.com/Cable/

## What The App Does

- Imports `.mp3` and `.wav` sound clips.
- Assigns clips to soundboard pads.
- Supports labels, global hotkeys, and per-pad volume.
- Routes microphone plus clips into Discord or game chat through VB-Cable.
- Lets you hear clips through your real speakers/headphones without monitoring your own mic.
- Applies toggleable mic-only DSP effects to the microphone path.
- Saves clips, layout, device choices, hotkeys, playback config, and effects locally.

## Documentation

Read these in order if you want to understand the project deeply:

1. [System Overview](docs/system-overview.md)
2. [Frontend Architecture](docs/frontend-architecture.md)
3. [Backend Architecture](docs/backend-architecture.md)
4. [Audio Engine](docs/audio-engine.md)
5. [Mic Effects And DSP](docs/mic-effects-and-dsp.md)
6. [Persistence](docs/persistence.md)
7. [Global Hotkeys](docs/global-hotkeys.md)
8. [Development Guide](docs/development-guide.md)
9. [Troubleshooting](docs/troubleshooting.md)

## Basic User Setup

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
