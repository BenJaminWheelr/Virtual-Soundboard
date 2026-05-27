# Frontend Architecture

The frontend is a React app written in TypeScript. It lives in `src/`.

The frontend does not do audio processing directly. Its job is to:

- Display the UI.
- Hold editable UI state.
- Validate simple user actions.
- Call Rust commands through Tauri.
- Render status and error messages.

## Entry Point

`src/main.tsx` mounts the React app.

`src/App.tsx` is the main application shell. It owns most frontend state and passes focused props into tab components.

## Main State In App.tsx

`App.tsx` tracks:

- Active tab.
- Busy/loading status.
- Top-bar message and message tone.
- Audio device lists.
- Current backend status.
- Selected microphone input.
- Selected monitor output.
- Uploaded clips.
- Soundboard grid size.
- Soundboard cells.
- Selected cell.
- Playback config.
- Mic effects config.
- Mic test state.

The key idea: child components are mostly controlled components. They display props and call callbacks. `App.tsx` decides what those callbacks do.

## Data Types

`src/types.ts` defines frontend data contracts such as:

- `SoundboardStatus`
- `AudioDeviceLists`
- `UploadedClip`
- `GridSize`
- `SoundboardCell`
- `SoundboardLayout`
- `MicEffectsConfig`

These types mirror the shapes exchanged with Rust through Tauri commands. For example, the frontend sends `MicEffectsConfig` to the backend, and Rust deserializes the same structure with Serde.

## Defaults

`src/defaults.ts` contains shared default values:

- `defaultGridSize`
- `defaultMicEffects`

Keeping defaults outside `App.tsx` prevents the app shell from becoming a dumping ground and makes defaults easy to reuse in tests or future setup flows.

## Soundboard State Helpers

`src/soundboardState.ts` contains:

- `gridDimensionOptions`
- `createCells`
- `clampCellVolume`

`createCells` is important. When the user changes the grid size, it preserves existing cells by ID where possible and creates new default cells for new positions.

Cell IDs follow this pattern:

```text
cell-0
cell-1
cell-2
...
```

That gives the layout stable positions across saves and grid changes.

## Tabs

The app uses simple tab routing:

- `main`
- `soundboard`
- `clips`
- `config`

The `Tabs` component changes the `activeTab` state in `App.tsx`. `App.tsx` conditionally renders the active tab.

## MainTab

`src/components/MainTab.tsx` handles:

- Microphone input selection.
- Monitor output selection.
- Device refresh.
- Audio engine start/stop.
- Mic test start/stop.
- VB-Cable status display.
- Optional audio stats terminal display.

It does not start the engine directly. It calls callbacks from `App.tsx`, and `App.tsx` calls Tauri commands.

## SoundboardTab

`src/components/SoundboardTab.tsx` handles:

- Displaying the soundboard grid.
- Selecting a cell.
- Opening the setup drawer.
- Editing cell label.
- Assigning/removing clips.
- Editing per-cell volume.
- Capturing hotkeys.
- Triggering pads.

Hotkey capture happens in a read-only input. When the user presses a key combination, the frontend formats it into a string like:

```text
Ctrl+Shift+A
Alt+Space
F
```

That string is stored on the selected `SoundboardCell`.

Pad clicks call `onPlayCell`. `App.playCell` handles validation:

- If no clip is assigned, show a red error message.
- If the audio engine is off, show a red error message.
- Otherwise invoke the backend `play_clip` command.

## AudioClipManagerTab

`src/components/AudioClipManagerTab.tsx` handles:

- Opening the file picker.
- Showing imported clips.
- Deleting clips.
- Displaying upload messages.

The actual file copy and decoding work happens in Rust.

## ConfigTab

`src/components/ConfigTab.tsx` handles:

- Clip monitor playback toggle.
- Clip boost toggle.
- Audio stats log visibility toggle.
- Mic effects controls.

The Effects section is built from small reusable UI helpers:

- `EffectSection`: toggle plus section wrapper.
- `EffectRange`: slider plus formatted output.

Current mic effects exposed in the UI:

- High-Pass Filter: cutoff.
- Low-Pass Filter: cutoff.
- Soft Saturation: drive.

Changing an effect calls `App.changeMicEffectsConfig`, which updates React state, sends the config to Rust, and saves the layout.

## Tauri Command Calls

The frontend uses:

```ts
import { invoke } from "@tauri-apps/api/core";
```

Examples:

```ts
await invoke("start_audio_engine", { selection });
await invoke("play_clip", { clipId, volume });
await invoke("set_mic_effects_config", { config });
```

The command name must match a `#[tauri::command]` function registered in `src-tauri/src/lib.rs`.

## Top-Bar Messages

`App.tsx` keeps:

```ts
const [message, setMessage] = useState("Backend idle");
const [messageTone, setMessageTone] = useState<"info" | "error">("info");
```

Use:

- `showInfo(...)` for normal progress/success.
- `showError(...)` for failures or invalid user actions.

The top bar renders error messages in red through CSS.

## Frontend Startup Flow

On mount, `App.tsx` calls `loadInitialData`.

That loads:

- Backend status.
- Audio devices.
- Clips.
- Saved layout.

Only after that finishes does the app render the tabs. Until then, it shows a loading panel.

## Frontend Save Flow

After the saved layout has loaded, layout-related state changes are auto-saved with a short debounce:

```text
cells
clipBoostEnabled
gridSize
monitorClipPlayback
micEffects
selectedInput
selectedMonitorOutput
```

The `layoutLoaded` guard prevents the initial empty/default state from overwriting the saved layout before loading completes.
