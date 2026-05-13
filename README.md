# Virtual Soundboard

Virtual Soundboard is a Windows desktop app for playing audio clips (Just like a normal physical soundboard). Hwoever, this program is, and always will be, free. The key feature of this program is that it works on top of other voice chat applications, such as discord. It does this by mixing audio clips directly with your microphone data, and sending it to the Virtual Cable output. Thus, when you set your input audio device to the Virtual Cable output, it is like the audio clip originated from your microphone. 

It is built with Tauri, Rust, TypeScript, and React.

## What It Does

- Import `.mp3` and `.wav` sound clips.
- Assign clips to soundboard pads.
- Add labels, hotkeys, and per-pad volume.
- Trigger clips from inside the app or with global hotkeys.
- Route your microphone plus soundboard clips into Discord or game chat through VB-Cable.
- Let you hear clips through your selected speakers/headphones without hearing your own mic.
- Save your layout, clips, device choices, hotkeys, and config locally.

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


### AirPods

Currently, Apple Airpods do not work with this application.


### My saved clips/layout are not included when I share the `.exe`

Correct. Saved clips, hotkeys, layout, and device choices are stored locally in your app data folder. Friends who run the app will start with their own fresh setup.
