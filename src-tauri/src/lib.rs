mod audio;
mod clip;
mod soundboard;

use serde::Deserialize;
use soundboard::{
    AudioDeviceLists, ClipRecord, DeviceSelection, HotkeyTarget, Soundboard, SoundboardStatus,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

struct AppState {
    soundboard: Mutex<Soundboard>,
    layout_path: PathBuf,
}

#[derive(Deserialize)]
struct HotkeyBinding {
    hotkey: String,
    clip_id: String,
    volume: f32,
}

#[derive(Deserialize, serde::Serialize)]
struct SoundboardLayout {
    grid_size: serde_json::Value,
    cells: serde_json::Value,
    ear_rape_enabled: Option<bool>,
    selected_input: Option<String>,
    selected_monitor_output: Option<String>,
    monitor_clip_playback: Option<bool>,
}

#[tauri::command]
fn backend_status(state: tauri::State<AppState>) -> Result<SoundboardStatus, String> {
    let soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    Ok(soundboard.status())
}

#[tauri::command]
fn list_audio_devices() -> Result<AudioDeviceLists, String> {
    soundboard::list_audio_devices()
}

#[tauri::command]
fn start_audio_engine(
    state: tauri::State<AppState>,
    selection: DeviceSelection,
) -> Result<SoundboardStatus, String> {
    let mut soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.start_audio_engine(selection)?;
    Ok(soundboard.status())
}

#[tauri::command]
fn stop_audio_engine(state: tauri::State<AppState>) -> Result<SoundboardStatus, String> {
    let mut soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.stop_audio_engine();
    Ok(soundboard.status())
}

#[tauri::command]
fn start_mic_test(
    state: tauri::State<AppState>,
    input_device: Option<String>,
) -> Result<(), String> {
    let mut soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.start_mic_test(input_device)
}

#[tauri::command]
fn stop_mic_test(state: tauri::State<AppState>) -> Result<(), String> {
    let mut soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.stop_mic_test();
    Ok(())
}

#[tauri::command]
fn mic_test_level(state: tauri::State<AppState>) -> Result<f32, String> {
    let soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    Ok(soundboard.mic_test_level())
}

#[tauri::command]
fn set_monitor_clip_playback(state: tauri::State<AppState>, enabled: bool) -> Result<(), String> {
    let mut soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.set_monitor_clip_playback(enabled);
    Ok(())
}

#[tauri::command]
fn set_ear_rape_enabled(state: tauri::State<AppState>, enabled: bool) -> Result<(), String> {
    let mut soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.set_ear_rape_enabled(enabled);
    Ok(())
}

#[tauri::command]
fn load_soundboard_layout(
    state: tauri::State<AppState>,
) -> Result<Option<SoundboardLayout>, String> {
    if !state.layout_path.exists() {
        return Ok(None);
    }

    let layout_json = fs::read_to_string(&state.layout_path).map_err(|err| err.to_string())?;
    serde_json::from_str(&layout_json).map_err(|err| err.to_string())
}

#[tauri::command]
fn save_soundboard_layout(
    state: tauri::State<AppState>,
    layout: SoundboardLayout,
) -> Result<(), String> {
    let parent = state
        .layout_path
        .parent()
        .ok_or("Layout path has no parent directory")?;
    fs::create_dir_all(parent).map_err(|err| err.to_string())?;

    let layout_json = serde_json::to_string_pretty(&layout).map_err(|err| err.to_string())?;
    fs::write(&state.layout_path, layout_json).map_err(|err| err.to_string())
}

#[tauri::command]
fn import_clip(state: tauri::State<AppState>, source_path: PathBuf) -> Result<ClipRecord, String> {
    let mut soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.import_clip(source_path)
}

#[tauri::command]
fn list_clips(state: tauri::State<AppState>) -> Result<Vec<ClipRecord>, String> {
    let mut soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.list_clips()
}

#[tauri::command]
fn delete_clip(state: tauri::State<AppState>, clip_id: String) -> Result<(), String> {
    let mut soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.delete_clip(&clip_id)
}

#[tauri::command]
fn play_clip(
    state: tauri::State<AppState>,
    clip_id: String,
    volume: f32,
) -> Result<SoundboardStatus, String> {
    let soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.play_clip(&clip_id, volume)?;
    Ok(soundboard.status())
}

#[tauri::command]
fn update_global_hotkeys(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    bindings: Vec<HotkeyBinding>,
) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|err| err.to_string())?;

    let mut unique_bindings = HashMap::<String, HotkeyTarget>::new();
    for binding in bindings {
        if binding.hotkey.trim().is_empty() || binding.clip_id.trim().is_empty() {
            continue;
        }

        unique_bindings.insert(
            binding.hotkey,
            HotkeyTarget {
                clip_id: binding.clip_id,
                volume: binding.volume,
            },
        );
    }

    let mut shortcuts = Vec::new();
    let mut hotkey_map = HashMap::new();

    for (hotkey, target) in unique_bindings {
        let shortcut = Shortcut::from_str(&hotkey)
            .map_err(|err| format!("Could not register hotkey '{hotkey}': {err}"))?;
        hotkey_map.insert(shortcut.id(), target);
        shortcuts.push(shortcut);
    }

    if !shortcuts.is_empty() {
        app.global_shortcut()
            .register_multiple(shortcuts)
            .map_err(|err| err.to_string())?;
    }

    let mut soundboard = state.soundboard.lock().map_err(|err| err.to_string())?;
    soundboard.set_hotkeys(hotkey_map);

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut: &Shortcut, event: ShortcutEvent| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }

                    let state = app.state::<AppState>();
                    let Ok(soundboard) = state.soundboard.lock() else {
                        return;
                    };

                    if let Err(err) = soundboard.play_hotkey(event.id) {
                        eprintln!("Global hotkey playback failed: {err}");
                    }
                })
                .build(),
        )
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let clips_dir = app_data_dir.join("clips");
            let layout_path = app_data_dir.join("soundboard-layout.json");
            app.manage(AppState {
                soundboard: Mutex::new(Soundboard::new(clips_dir)),
                layout_path,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            backend_status,
            list_audio_devices,
            start_audio_engine,
            stop_audio_engine,
            start_mic_test,
            stop_mic_test,
            mic_test_level,
            set_ear_rape_enabled,
            set_monitor_clip_playback,
            load_soundboard_layout,
            save_soundboard_layout,
            import_clip,
            list_clips,
            delete_clip,
            play_clip,
            update_global_hotkeys
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
