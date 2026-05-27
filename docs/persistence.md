# Persistence

The app persists data through the Rust backend, not through browser `localStorage`.

Tauri gives the backend an app data directory. The backend stores imported clips and layout/config there.

## Stored Files

The backend creates:

```text
app_data_dir\clips\
app_data_dir\soundboard-layout.json
```

`clips` contains copied `.mp3` and `.wav` files.

`soundboard-layout.json` contains user-editable configuration.

## App Data Directory

In `src-tauri/src/lib.rs`, setup calls:

```rust
let app_data_dir = app.path().app_data_dir()?;
```

Then:

```rust
let clips_dir = app_data_dir.join("clips");
let layout_path = app_data_dir.join("soundboard-layout.json");
```

These paths are stored in `AppState` and `Soundboard`.

## Layout JSON

The frontend sends a `SoundboardLayout` to Rust.

It includes:

- Grid size.
- Soundboard cells.
- Clip boost setting.
- Monitor clip playback setting.
- Selected input device.
- Selected monitor output device.
- Mic effects config.
- Whether the audio stats log is visible.

Each cell includes:

- Cell ID.
- Label.
- Assigned clip ID.
- Hotkey string.
- Volume.

## Loading Layout

Startup flow:

```text
App.tsx loadInitialData
-> loadSoundboardLayout
-> invoke("load_soundboard_layout")
-> lib.rs load_soundboard_layout
-> read soundboard-layout.json
-> deserialize JSON
-> return layout to frontend
```

If the file does not exist, Rust returns `None`. The frontend keeps defaults.

## Saving Layout

`App.tsx` saves layout in two ways:

1. Some settings save immediately after a user action.
2. A debounced effect saves after layout-related state changes.

The debounced state list includes:

- `cells`
- `clipBoostEnabled`
- `gridSize`
- `monitorClipPlayback`
- `micEffects`
- `selectedInput`
- `selectedMonitorOutput`

The debounce avoids writing the layout file on every tiny slider movement or rapid state change.

## layoutLoaded Guard

`layoutLoaded` prevents accidental overwrites.

Without this guard, React could mount with default values and immediately save them before the real saved layout finishes loading.

The save effect starts only after layout loading has completed.

## Importing Clips

Clip import flow:

```text
Frontend file dialog
-> selected file paths
-> invoke("import_clip")
-> Soundboard.import_clip
-> copy source file into app_data_dir\clips
-> decode file into memory
-> return ClipRecord
```

The source file remains where it was. The app uses its copied version after import.

## Clip IDs

Imported clips receive IDs based on the file stem plus timestamp.

This reduces collisions and gives stable IDs for cell assignments.

The stored file name looks like:

```text
clip-id.extension
```

The frontend stores only `clipId` on a soundboard cell. On restart, Rust reloads clips from disk and recreates `ClipRecord` values.

## Listing Clips

`list_clips` calls `load_persisted_clips`.

That function:

1. Ensures the clips directory exists.
2. Reads files in the clips directory.
3. Ignores unsupported extensions.
4. Builds clip records.
5. Decodes any clip not already in memory.
6. Sorts records by display name.

## Deleting Clips

Deleting a clip:

1. Removes it from the in-memory clip map.
2. Searches the clips directory for matching files.
3. Deletes matching files.
4. Frontend removes the clip from the UI.
5. Frontend clears any cells that referenced the deleted clip.

## What Is Not Shared With The EXE

Saved clips and layout are not baked into the app binary.

If you send someone the built `.exe`, they get the program, not your local app data.

Their app creates its own app data directory and starts fresh.
