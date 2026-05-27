# Troubleshooting

## VB-Cable Says It Is Not Installed

Make sure Windows shows an output device named something like:

```text
CABLE Input (VB-Audio Virtual Cable)
```

Then restart the app or press **Scan** on the Main tab.

The app writes audio to `CABLE Input`. Discord/game should listen to `CABLE Output`.

## I Do Not Hear My Clips

Check:

- The audio engine is running.
- The pad has a clip assigned.
- The Config tab has **Hear my audio clips** enabled.
- The monitor output is your real speakers/headphones, not VB-Cable.
- The per-pad volume is not very low.
- The clip imported successfully.

## Other People Do Not Hear My Clips

Check:

- Discord/game input is set to `CABLE Output (VB-Audio Virtual Cable)`.
- The audio engine is running.
- VB-Cable is installed and detected.
- Voice chat noise suppression is not filtering clips.

Recommended voice chat settings:

- Turn off echo cancellation.
- Turn off noise suppression.
- Lower the voice chat noise gate as far as possible.
- Disable automatic input sensitivity if it cuts clips off.

## Nobody Hears My Mic

Check that the Main tab input device is your actual microphone.

Also make sure Discord/game input is `CABLE Output`, not your physical microphone.

## I Hear Myself

The app should not monitor your microphone through the monitor output. If you hear yourself, check whether Windows, Discord, your headset software, or another app has mic monitoring enabled.

## Effects Are Not Changing Clips

That is expected.

Mic effects are applied only to live microphone audio. Soundboard clips remain clean.

## Hotkeys Do Nothing

Check:

- The audio engine is running.
- The cell has a clip assigned.
- The cell has a hotkey assigned.
- Another app is not already using the same shortcut.

Hotkeys are intentionally unregistered while the audio engine is off.

## Clicking A Pad Shows An Error

The top status text turns red for error messages.

Common errors:

- `Assign a clip to this cell first`
- `Start the audio engine before playing clips`

These are frontend validation messages.

## AirPods

Currently, Apple AirPods do not work with this application.

## Saved Clips/Layout Are Not Included With The EXE

Correct.

Saved clips, hotkeys, layout, device choices, playback settings, and effects are stored locally in the user's app data folder. They are not bundled into the executable.

## Build Fails Because Of Generated Assets

`npm run build` regenerates files in `dist/`.

Hashed files can change names:

```text
dist/assets/index-oldhash.js
dist/assets/index-newhash.js
```

That is normal.

## Rust Audio Device Errors

Audio device errors usually mean:

- The selected device disappeared.
- The device is in exclusive use.
- The default format could not be opened.
- VB-Cable is missing.

Try rescanning devices, restarting the app, or restarting the audio device.
