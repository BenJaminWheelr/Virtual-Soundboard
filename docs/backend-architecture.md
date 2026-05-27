# Backend Architecture

The backend is a Rust library used by Tauri. It lives in `src-tauri/src/`.

Its responsibilities are:

- Register Tauri commands.
- Own shared app state.
- Read and write local files.
- Discover audio devices.
- Start and stop audio streams.
- Decode audio clips.
- Mix mic audio and clips.
- Register global hotkeys.
- Apply mic DSP effects.

## Tauri Entry Point

`src-tauri/src/main.rs` is tiny. It calls into the library:

```rust
virtual_soundboard_lib::run();
```

The real setup happens in `src-tauri/src/lib.rs`.

## AppState

`lib.rs` defines:

```rust
struct AppState {
    soundboard: Mutex<Soundboard>,
    layout_path: PathBuf,
}
```

`soundboard` is wrapped in a `Mutex` because multiple Tauri commands can access shared backend state. The mutex allows one command at a time to read or mutate the `Soundboard`.

`layout_path` points to:

```text
app_data_dir\soundboard-layout.json
```

The app data directory is resolved by Tauri:

```rust
let app_data_dir = app.path().app_data_dir()?;
```

## Setup

In `run`, Tauri registers:

- The dialog plugin.
- The global shortcut plugin.
- The shared `AppState`.
- The command handler list.

The setup block creates:

```text
clips_dir = app_data_dir\clips
layout_path = app_data_dir\soundboard-layout.json
```

Then it creates:

```rust
Soundboard::new(clips_dir)
```

## Commands

Every function marked with `#[tauri::command]` can be called by the frontend with `invoke`.

Example:

```rust
#[tauri::command]
fn start_audio_engine(
    state: tauri::State<AppState>,
    selection: DeviceSelection,
) -> Result<SoundboardStatus, String> {
    let mut soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.start_audio_engine(selection)?;
    Ok(soundboard.status())
}
```

This function:

1. Locks the shared `Soundboard`.
2. Starts the audio engine.
3. Returns the new status.
4. Uses `?` to return early if an error occurs.

The frontend calls it like this:

```ts
await invoke("start_audio_engine", { selection });
```

## Serialization

Rust uses Serde to convert frontend payloads into Rust structs.

Example:

```rust
#[derive(Deserialize)]
pub struct DeviceSelection {
    pub input_device: Option<String>,
    pub monitor_output_device: Option<String>,
}
```

`#[derive(Deserialize)]` generates code that lets Tauri parse incoming JSON-like data into `DeviceSelection`.

Rust structs returned to the frontend use `Serialize`.

## Soundboard

`src-tauri/src/soundboard.rs` defines the central backend model:

```rust
pub struct Soundboard {
    engine: Option<AudioEngine>,
    mic_test: Option<MicTest>,
    clips: HashMap<String, Arc<AudioClip>>,
    clips_dir: PathBuf,
    hotkeys: HashMap<u32, HotkeyTarget>,
    clip_boost_enabled: bool,
    monitor_clip_playback: bool,
    mic_effects: SharedMicEffects,
}
```

Important fields:

- `engine`: running audio streams, or `None` when stopped.
- `mic_test`: temporary mic-level stream.
- `clips`: decoded clip data in memory.
- `clips_dir`: persisted imported clip directory.
- `hotkeys`: maps global shortcut IDs to clip targets.
- `mic_effects`: shared real-time-safe mic effects config.

## Clip Management

`Soundboard.import_clip`:

1. Validates the extension is `.mp3` or `.wav`.
2. Creates the clips directory if needed.
3. Builds a unique clip ID from the file name and timestamp.
4. Copies the file into the app data clips directory.
5. Decodes it into an `AudioClip`.
6. Stores it in memory.
7. Returns a `ClipRecord` to the frontend.

`Soundboard.list_clips` calls `load_persisted_clips`, which scans the clips directory and decodes any supported files not already loaded.

`Soundboard.delete_clip` removes the clip from memory and deletes matching files from disk.

## Audio Engine Lifecycle

`Soundboard.start_audio_engine`:

1. Returns early if already running.
2. Loads persisted clips.
3. Starts `AudioEngine`.
4. Stores it in `self.engine`.

`Soundboard.stop_audio_engine`:

```rust
self.engine = None;
```

Dropping the `AudioEngine` drops the CPAL streams it owns, which stops audio processing.

## Mic Test

The mic test is separate from the main audio engine.

`start_mic_test` opens an input stream and tracks peak level in an atomic float representation. The frontend polls `mic_test_level` every 60ms while the test is running.

The app stops the mic test before starting the full audio engine. This prevents the same input device from being used by competing streams.

## Global Hotkeys

The backend uses `tauri-plugin-global-shortcut`.

`update_global_hotkeys`:

1. Unregisters all existing shortcuts.
2. Deduplicates frontend bindings by hotkey string.
3. Parses strings into Tauri `Shortcut` values.
4. Registers them with the OS.
5. Stores a map from shortcut ID to clip target.

`clear_global_hotkeys`:

1. Unregisters all shortcuts.
2. Clears the in-memory hotkey map.

The plugin handler reacts only to `ShortcutState::Pressed`. It looks up the event ID in the soundboard hotkey map and plays the corresponding clip.

## Audio Stats Log

The audio engine tracks dropped input frames and missing output frames with atomics. A lightweight stats thread samples those counters every two seconds.

The stats thread:

1. Swaps the current counters back to zero.
2. Appends command-line-style warning lines to a small rolling log only when counters are non-zero.
3. Prints warnings to stderr when counters are non-zero.

The frontend reads the rolling log through the `audio_stats_log` command. The log is shown on the Main tab only when enabled in Config. The rolling log is stored on `Soundboard`, so it survives audio engine stop/start within the current app session.

## Error Handling

Most commands return:

```rust
Result<T, String>
```

This keeps frontend error handling simple. Backend errors are converted into strings with:

```rust
.map_err(|err| err.to_string())?
```

The `?` operator means:

- If `Ok(value)`, unwrap and continue.
- If `Err(error)`, return that error immediately.

## Module Responsibilities

- `lib.rs`: command boundary and Tauri setup.
- `soundboard.rs`: high-level backend state and lifecycle.
- `audio.rs`: real-time stream construction and mixing.
- `clip.rs`: audio file decoding and clip sample playback.
- `effects/mod.rs`: shared effects config and effect-chain orchestration.
- `effects/simple.rs`: small stateless/stateful simple DSP blocks.
