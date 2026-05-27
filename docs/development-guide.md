# Development Guide

This guide is for working on the project without having to rediscover the architecture every time.

## Install

```powershell
npm install
```

## Run

```powershell
npm run tauri dev
```

This starts the Vite frontend and launches the Tauri desktop app.

## Build

```powershell
npm run tauri build
```

The release executable is under:

```text
src-tauri\target\release\
```

Installers, if generated, are under:

```text
src-tauri\target\release\bundle\
```

## Verification Commands

Frontend build:

```powershell
npm run build
```

Rust check:

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
```

Rust format:

```powershell
cargo fmt --manifest-path src-tauri\Cargo.toml
```

## Recommended Reading Order

If you are new to the project, read code in this order:

1. `package.json`
2. `src-tauri/Cargo.toml`
3. `src/types.ts`
4. `src/defaults.ts`
5. `src/App.tsx`
6. `src/components/MainTab.tsx`
7. `src/components/SoundboardTab.tsx`
8. `src/components/AudioClipManagerTab.tsx`
9. `src/components/ConfigTab.tsx`
10. `src/soundboardState.ts`
11. `src-tauri/src/lib.rs`
12. `src-tauri/src/soundboard.rs`
13. `src-tauri/src/audio.rs`
14. `src-tauri/src/clip.rs`
15. `src-tauri/src/effects/mod.rs`
16. `src-tauri/src/effects/simple.rs`

## Adding A Frontend Setting

For a new user-facing setting:

1. Add a TypeScript type in `src/types.ts`.
2. Add a default in `src/defaults.ts` if needed.
3. Add React state or extend existing state in `App.tsx`.
4. Add UI controls in the relevant component.
5. Send changes to Rust with `invoke` if the backend needs them.
6. Include the setting in `SoundboardLayout` if it should persist.
7. Update docs.

## Adding A Backend Command

1. Add a `#[tauri::command]` function in `src-tauri/src/lib.rs`.
2. Accept `tauri::State<AppState>` if the command needs shared state.
3. Lock the soundboard only for as long as needed.
4. Return `Result<T, String>`.
5. Add the command to `tauri::generate_handler![...]`.
6. Call it from the frontend with `invoke`.
7. Update docs.

## Adding A Mic Effect

Use the existing effect architecture.

1. Add Rust config fields in `src-tauri/src/effects/mod.rs`.
2. Add TypeScript config fields in `src/types.ts`.
3. Add defaults in `src/defaults.ts`.
4. Add UI controls in `ConfigTab.tsx`.
5. Store shared values in `SharedMicEffects` using atomics.
6. Add fields to `MicEffectsSnapshot`.
7. Implement DSP in `effects/simple.rs` or a new module.
8. Call it from `MicEffectsProcessor.process_sample`.
9. Run `cargo check`, `cargo fmt`, and `npm run build`.
10. Update `docs/mic-effects-and-dsp.md`.

## Adding A Clip Feature

Clip features usually touch:

- `src/components/AudioClipManagerTab.tsx`
- `src/App.tsx`
- `src-tauri/src/soundboard.rs`
- `src-tauri/src/clip.rs`

If the feature changes clip metadata, consider whether it must be persisted in the layout JSON, the clip file name, or a future separate clip metadata file.

## Adding A Soundboard Cell Feature

Cell features usually touch:

- `src/types.ts`
- `src/soundboardState.ts`
- `src/components/SoundboardTab.tsx`
- `src/App.tsx`
- `src-tauri/src/lib.rs` if sent to backend

Remember that `createCells` should preserve existing cell state when grid dimensions change.

## Build Artifacts

`npm run build` writes to `dist/` and uses hashed asset names. That means old asset files can be deleted and new asset files can appear after each build.

Do not confuse those generated changes with source code changes.

## Safety Notes

Be careful with:

- Audio callbacks: avoid blocking work.
- Global hotkeys: always unregister stale shortcuts.
- Persistence: do not save defaults before loading the saved layout.
- Device names: users may have different devices and sample formats.
- Clip deletion: clear cell references after deleting a clip.
