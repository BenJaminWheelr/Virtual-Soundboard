# Global Hotkeys

Global hotkeys let the user trigger soundboard pads while another app is focused.

The project uses:

- Rust plugin: `tauri-plugin-global-shortcut`
- Frontend hotkey capture in `SoundboardTab.tsx`
- Backend registration in `src-tauri/src/lib.rs`

## Important Behavior

Hotkeys are registered with the operating system only while the audio engine is running.

This means:

- Start engine: register current hotkeys.
- Edit hotkeys while engine is running: re-register hotkeys.
- Stop engine: unregister all hotkeys.

This prevents stale global shortcuts from staying active when playback cannot happen.

## Frontend Capture

The user edits a cell hotkey in `SoundboardTab.tsx`.

The hotkey input is read-only. When a key combination is pressed, `formatHotkey` creates a string:

```text
Ctrl+Shift+A
Alt+Space
F
```

The string is stored on:

```ts
SoundboardCell.hotkey
```

The cell also needs a `clipId`; hotkeys without clips are ignored by registration.

## Frontend Registration Trigger

`App.tsx` has an effect that watches cells and engine status:

```text
if engine is running:
    updateGlobalHotkeys()
```

`updateGlobalHotkeys` builds bindings:

```ts
{
  hotkey: cell.hotkey,
  clip_id: cell.clipId,
  volume: cell.volume
}
```

Then it calls:

```ts
invoke("update_global_hotkeys", { bindings });
```

## Backend Registration

`update_global_hotkeys` in `lib.rs`:

1. Calls `unregister_all`.
2. Filters empty bindings.
3. Deduplicates by hotkey string.
4. Parses each string into a Tauri `Shortcut`.
5. Registers all shortcuts.
6. Stores a `HashMap` from shortcut ID to `HotkeyTarget`.

`HotkeyTarget` stores:

```rust
pub struct HotkeyTarget {
    pub clip_id: String,
    pub volume: f32,
}
```

The key detail: the OS shortcut event gives the backend a shortcut ID, not the frontend cell. The backend uses that ID to find the clip target.

## Handling A Hotkey Press

The global shortcut plugin is configured in `lib.rs`:

```rust
tauri_plugin_global_shortcut::Builder::new()
    .with_handler(...)
```

The handler:

1. Ignores anything that is not `ShortcutState::Pressed`.
2. Locks `AppState.soundboard`.
3. Calls `soundboard.play_hotkey(event.id)`.

`play_hotkey`:

1. Looks up the ID in the hotkey map.
2. If found, calls `play_clip`.
3. If not found, does nothing.

## Clearing Hotkeys

When the audio engine stops, `App.stopAudioEngine` calls:

```ts
invoke("clear_global_hotkeys");
```

The Rust command:

1. Calls `unregister_all`.
2. Clears the soundboard hotkey map.

## Duplicate Hotkeys

The backend deduplicates with:

```rust
HashMap::<String, HotkeyTarget>
```

If multiple cells use the same hotkey, the last inserted binding for that hotkey wins.

This is simple but not very user-visible. A future improvement could detect duplicates in the UI and show a warning.

## Failure Cases

Registration can fail if:

- The hotkey string cannot be parsed.
- Another app already owns the shortcut.
- The OS rejects the shortcut.

The backend returns an error string. The frontend displays it in the top message area as an error.

## Mental Model

Think of the hotkey system as two maps:

Frontend:

```text
cell -> hotkey string + clip ID + volume
```

Backend:

```text
registered shortcut ID -> clip ID + volume
```

The frontend owns editing. The backend owns OS registration and playback.
