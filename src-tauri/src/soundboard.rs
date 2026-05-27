use crate::audio::{
    AudioCommand, AudioStats, AudioStatsLog, VoiceOutputStreamParts, build_clip_monitor_stream,
    build_input_stream, build_output_stream,
};
use crate::clip::AudioClip;
use crate::effects::{MicEffectsConfig, SharedMicEffects};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};
use ringbuf::{
    HeapRb,
    traits::{Producer, Split},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
    mpsc::{self, Sender},
};
use std::thread;
use std::time::Duration;

const LATENCY_MS: f32 = 80.0;

pub struct Soundboard {
    engine: Option<AudioEngine>,
    mic_test: Option<MicTest>,
    clips: HashMap<String, Arc<AudioClip>>,
    clips_dir: PathBuf,
    hotkeys: HashMap<u32, HotkeyTarget>,
    clip_boost_enabled: bool,
    monitor_clip_playback: bool,
    mic_effects: SharedMicEffects,
    stats_log: AudioStatsLog,
}

impl Soundboard {
    pub fn new(clips_dir: PathBuf) -> Self {
        Self {
            engine: None,
            mic_test: None,
            clips: HashMap::new(),
            clips_dir,
            hotkeys: HashMap::new(),
            clip_boost_enabled: false,
            monitor_clip_playback: true,
            mic_effects: SharedMicEffects::new(),
            stats_log: AudioStatsLog::new(),
        }
    }

    pub fn start_audio_engine(&mut self, selection: DeviceSelection) -> Result<(), String> {
        if self.engine.is_some() {
            return Ok(());
        }

        self.load_persisted_clips()?;
        self.stats_log.push("$ audio engine starting".into());
        let engine =
            AudioEngine::start(selection, self.mic_effects.clone(), self.stats_log.clone())
                .map_err(|err| err.to_string())?;

        self.engine = Some(engine);
        self.stats_log.push("$ audio engine running".into());
        Ok(())
    }

    pub fn stop_audio_engine(&mut self) {
        if self.engine.is_some() {
            self.stats_log.push("$ audio engine stopping".into());
            self.engine = None;
            self.stats_log.push("$ audio engine stopped".into());
        }
    }

    pub fn start_mic_test(&mut self, input_device: Option<String>) -> Result<(), String> {
        if self.mic_test.is_some() {
            return Ok(());
        }

        let mic_test = MicTest::start(input_device).map_err(|err| err.to_string())?;
        self.mic_test = Some(mic_test);
        Ok(())
    }

    pub fn stop_mic_test(&mut self) {
        self.mic_test = None;
    }

    pub fn mic_test_level(&self) -> f32 {
        self.mic_test
            .as_ref()
            .map(|mic_test| mic_test.level())
            .unwrap_or(0.0)
    }

    pub fn import_clip(&mut self, source_path: PathBuf) -> Result<ClipRecord, String> {
        let extension = source_path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .ok_or("Audio clip path has no extension")?;

        if extension != "mp3" && extension != "wav" {
            return Err("Only MP3 and WAV clips are supported".into());
        }

        fs::create_dir_all(&self.clips_dir).map_err(|err| err.to_string())?;

        let name = source_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("Audio Clip")
            .to_string();
        let id = create_clip_id(&name);
        let file_name = format!("{id}.{extension}");
        let stored_path = self.clips_dir.join(&file_name);

        fs::copy(&source_path, &stored_path).map_err(|err| err.to_string())?;

        let clip = Arc::new(
            AudioClip::load(&stored_path)
                .map_err(|err| format!("Failed to decode imported clip: {err}"))?,
        );
        self.clips.insert(id.clone(), clip);

        Ok(ClipRecord {
            id,
            name,
            file_name,
            format: extension,
            path: stored_path.display().to_string(),
        })
    }

    pub fn list_clips(&mut self) -> Result<Vec<ClipRecord>, String> {
        self.load_persisted_clips()
    }

    pub fn delete_clip(&mut self, clip_id: &str) -> Result<(), String> {
        self.clips.remove(clip_id);
        if !self.clips_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.clips_dir).map_err(|err| err.to_string())? {
            let path = entry.map_err(|err| err.to_string())?.path();
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };

            if clip_id_from_stem(stem) == clip_id {
                fs::remove_file(path).map_err(|err| err.to_string())?;
            }
        }

        Ok(())
    }

    pub fn play_clip(&self, clip_id: &str, volume: f32) -> Result<(), String> {
        let engine = self
            .engine
            .as_ref()
            .ok_or("Start the audio engine before playing clips")?;
        let clip = self.clips.get(clip_id).ok_or("Clip has not been loaded")?;

        let playback_volume = if self.clip_boost_enabled {
            volume * 15.0
        } else {
            volume
        };

        engine.play_clip(
            Arc::clone(clip),
            playback_volume,
            self.monitor_clip_playback,
        )
    }

    pub fn set_clip_boost_enabled(&mut self, enabled: bool) {
        self.clip_boost_enabled = enabled;
    }

    pub fn set_monitor_clip_playback(&mut self, enabled: bool) {
        self.monitor_clip_playback = enabled;
    }

    pub fn mic_effects_config(&self) -> MicEffectsConfig {
        self.mic_effects.config()
    }

    pub fn set_mic_effects_config(&self, config: MicEffectsConfig) {
        self.mic_effects.set_config(config);
    }

    pub fn set_hotkeys(&mut self, hotkeys: HashMap<u32, HotkeyTarget>) {
        self.hotkeys = hotkeys;
    }

    pub fn play_hotkey(&self, hotkey_id: u32) -> Result<(), String> {
        let Some(target) = self.hotkeys.get(&hotkey_id) else {
            return Ok(());
        };

        self.play_clip(&target.clip_id, target.volume)
    }

    pub fn status(&self) -> SoundboardStatus {
        SoundboardStatus {
            engine_running: self.engine.is_some(),
            clips_dir: self.clips_dir.display().to_string(),
            clip_count: self.clips.len(),
        }
    }

    pub fn audio_stats_log(&self) -> Vec<String> {
        let lines = self.stats_log.lines();
        if lines.is_empty() {
            vec!["$ audio engine stopped".into()]
        } else {
            lines
        }
    }

    fn load_persisted_clips(&mut self) -> Result<Vec<ClipRecord>, String> {
        fs::create_dir_all(&self.clips_dir).map_err(|err| err.to_string())?;
        let mut records = Vec::new();

        for entry in fs::read_dir(&self.clips_dir).map_err(|err| err.to_string())? {
            let path = entry.map_err(|err| err.to_string())?.path();
            if !is_supported_clip_path(&path) {
                continue;
            }

            let record = clip_record_from_path(&path)?;
            if !self.clips.contains_key(&record.id) {
                let clip = Arc::new(
                    AudioClip::load(&path)
                        .map_err(|err| format!("Failed to decode {}: {err}", record.file_name))?,
                );
                self.clips.insert(record.id.clone(), clip);
            }
            records.push(record);
        }

        records.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(records)
    }
}

#[derive(Deserialize)]
pub struct DeviceSelection {
    pub input_device: Option<String>,
    pub monitor_output_device: Option<String>,
}

pub struct HotkeyTarget {
    pub clip_id: String,
    pub volume: f32,
}

#[derive(Serialize)]
pub struct AudioDeviceLists {
    pub inputs: Vec<AudioDeviceInfo>,
    pub outputs: Vec<AudioDeviceInfo>,
    pub vb_cable: VbCableStatus,
    pub monitor_output: Option<AudioDeviceInfo>,
}

#[derive(Serialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub channels: u16,
    pub sample_rate: u32,
    pub sample_format: String,
}

#[derive(Serialize)]
pub struct VbCableStatus {
    pub installed: bool,
    pub playback_device: Option<AudioDeviceInfo>,
    pub voice_chat_input_name: String,
}

#[derive(Serialize)]
pub struct SoundboardStatus {
    pub engine_running: bool,
    pub clips_dir: String,
    pub clip_count: usize,
}

#[derive(Serialize)]
pub struct ClipRecord {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub format: String,
    pub path: String,
}

pub fn list_audio_devices() -> Result<AudioDeviceLists, String> {
    let host = cpal::default_host();
    let inputs = host
        .input_devices()
        .map_err(|err| err.to_string())?
        .filter_map(|device| input_device_info(&device).ok())
        .collect();
    let outputs = host
        .output_devices()
        .map_err(|err| err.to_string())?
        .filter(|device| !is_vb_cable_device(device))
        .filter_map(|device| output_device_info(&device).ok())
        .collect();

    let vb_cable = vb_cable_status(&host)?;
    let monitor_output = host
        .default_output_device()
        .filter(|device| !is_vb_cable_device(device))
        .or_else(|| first_non_vb_cable_output_device(&host).ok().flatten())
        .and_then(|device| output_device_info(&device).ok());

    Ok(AudioDeviceLists {
        inputs,
        outputs,
        vb_cable,
        monitor_output,
    })
}

fn input_device_info(device: &cpal::Device) -> Result<AudioDeviceInfo, Box<dyn std::error::Error>> {
    let name = device.description()?.name().to_string();
    let config = device.default_input_config()?;
    Ok(audio_device_info(name, &config))
}

fn output_device_info(
    device: &cpal::Device,
) -> Result<AudioDeviceInfo, Box<dyn std::error::Error>> {
    let name = device.description()?.name().to_string();
    let config = device.default_output_config()?;
    Ok(audio_device_info(name, &config))
}

fn audio_device_info(name: String, config: &cpal::SupportedStreamConfig) -> AudioDeviceInfo {
    let stream_config: cpal::StreamConfig = config.clone().into();
    AudioDeviceInfo {
        name,
        channels: stream_config.channels,
        sample_rate: stream_config.sample_rate,
        sample_format: config.sample_format().to_string(),
    }
}

struct AudioEngine {
    voice_command_sender: Sender<AudioCommand>,
    monitor_command_sender: Sender<AudioCommand>,
    _input_stream: cpal::Stream,
    _voice_stream: cpal::Stream,
    _monitor_stream: cpal::Stream,
    stats_thread_running: Arc<AtomicBool>,
}

struct MicTest {
    level: Arc<AtomicU32>,
    _stream: cpal::Stream,
}

impl MicTest {
    fn start(input_device_name: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let input_device = find_input_device(&host, input_device_name)?
            .or_else(|| host.default_input_device())
            .ok_or("Failed to get default input device")?;
        let supported_config = input_device.default_input_config()?;
        let config: cpal::StreamConfig = supported_config.clone().into();
        let input_channels = config.channels as usize;
        let level = Arc::new(AtomicU32::new(0.0f32.to_bits()));

        let stream = build_mic_test_stream(
            &input_device,
            &config,
            supported_config.sample_format(),
            input_channels,
            Arc::clone(&level),
        )?;
        stream.play()?;

        Ok(Self {
            level,
            _stream: stream,
        })
    }

    fn level(&self) -> f32 {
        f32::from_bits(self.level.load(Ordering::Relaxed))
    }
}

fn build_mic_test_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: SampleFormat,
    input_channels: usize,
    level: Arc<AtomicU32>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    match sample_format {
        SampleFormat::I8 => {
            build_typed_mic_test_stream::<i8>(device, config, input_channels, level)
        }
        SampleFormat::I16 => {
            build_typed_mic_test_stream::<i16>(device, config, input_channels, level)
        }
        SampleFormat::I24 => {
            build_typed_mic_test_stream::<cpal::I24>(device, config, input_channels, level)
        }
        SampleFormat::I32 => {
            build_typed_mic_test_stream::<i32>(device, config, input_channels, level)
        }
        SampleFormat::I64 => {
            build_typed_mic_test_stream::<i64>(device, config, input_channels, level)
        }
        SampleFormat::U8 => {
            build_typed_mic_test_stream::<u8>(device, config, input_channels, level)
        }
        SampleFormat::U16 => {
            build_typed_mic_test_stream::<u16>(device, config, input_channels, level)
        }
        SampleFormat::U24 => {
            build_typed_mic_test_stream::<cpal::U24>(device, config, input_channels, level)
        }
        SampleFormat::U32 => {
            build_typed_mic_test_stream::<u32>(device, config, input_channels, level)
        }
        SampleFormat::U64 => {
            build_typed_mic_test_stream::<u64>(device, config, input_channels, level)
        }
        SampleFormat::F32 => {
            build_typed_mic_test_stream::<f32>(device, config, input_channels, level)
        }
        SampleFormat::F64 => {
            build_typed_mic_test_stream::<f64>(device, config, input_channels, level)
        }
        format => panic!("Unsupported input sample format: {format:?}"),
    }
}

fn build_typed_mic_test_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    input_channels: usize,
    level: Arc<AtomicU32>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: Sample + SizedSample,
    f32: FromSample<T>,
{
    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut peak = 0.0f32;

            for frame in data.chunks(input_channels) {
                let mono = frame
                    .iter()
                    .map(|sample| sample.to_sample::<f32>())
                    .sum::<f32>()
                    / frame.len() as f32;
                peak = peak.max(mono.abs());
            }

            let previous = f32::from_bits(level.load(Ordering::Relaxed));
            let smoothed = if peak > previous {
                previous + (peak - previous) * 0.45
            } else {
                previous * 0.82
            };
            level.store(smoothed.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
        },
        |err| eprintln!("Mic test stream error: {err}"),
        None,
    )
}

impl AudioEngine {
    fn start(
        selection: DeviceSelection,
        mic_effects: SharedMicEffects,
        stats_log: AudioStatsLog,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let input_device = find_input_device(&host, selection.input_device)?
            .or_else(|| host.default_input_device())
            .ok_or("Failed to get default input device")?;
        let voice_output_device = find_vb_cable_playback_device(&host)?
            .ok_or("VB-Cable was not found. Install VB-Cable, restart the app, then choose 'CABLE Output (VB-Audio Virtual Cable)' as the microphone/input in Discord or your game.")?;
        let monitor_output_device =
            find_monitor_output_device(&host, selection.monitor_output_device)?
                .or_else(|| {
                    host.default_output_device()
                        .filter(|device| !is_vb_cable_device(device))
                })
                .or_else(|| first_non_vb_cable_output_device(&host).ok().flatten())
                .ok_or("Failed to get default output device for clip monitoring")?;

        let input_supported_config = input_device.default_input_config()?;
        let voice_supported_config = voice_output_device.default_output_config()?;
        let monitor_supported_config = monitor_output_device.default_output_config()?;
        let input_config: cpal::StreamConfig = input_supported_config.clone().into();
        let voice_config: cpal::StreamConfig = voice_supported_config.clone().into();
        let monitor_config: cpal::StreamConfig = monitor_supported_config.clone().into();

        let input_sample_rate = input_config.sample_rate as f32;
        let voice_sample_rate = voice_config.sample_rate as f32;
        let input_channels = input_config.channels as usize;
        let voice_channels = voice_config.channels as usize;
        let monitor_channels = monitor_config.channels as usize;

        let latency_frames = ((LATENCY_MS / 1_000.0) * input_sample_rate).round() as usize;
        let rb = HeapRb::<f32>::new((latency_frames * 4).max(1024));
        let (mut producer, consumer) = rb.split();
        prefill_silence(&mut producer, latency_frames);

        let stats = AudioStats::new(stats_log);
        stats.log.push("$ audio stats logger initialized".into());
        let (voice_command_sender, voice_command_receiver) = mpsc::channel();
        let (monitor_command_sender, monitor_command_receiver) = mpsc::channel();

        let input_stream = build_input_stream(
            &input_device,
            &input_config,
            input_supported_config.sample_format(),
            input_channels,
            producer,
            stats.dropped_input_frames.clone(),
            mic_effects,
        )?;

        let voice_stream = build_output_stream(VoiceOutputStreamParts {
            device: &voice_output_device,
            config: &voice_config,
            sample_format: voice_supported_config.sample_format(),
            output_channels: voice_channels,
            mic_resample_step: input_sample_rate / voice_sample_rate,
            command_receiver: voice_command_receiver,
            consumer,
            missing_frames: stats.missing_output_frames.clone(),
        })?;

        let monitor_stream = build_clip_monitor_stream(
            &monitor_output_device,
            &monitor_config,
            monitor_supported_config.sample_format(),
            monitor_channels,
            monitor_command_receiver,
        )?;

        input_stream.play()?;
        voice_stream.play()?;
        monitor_stream.play()?;

        let stats_thread_running = start_stats_logger(stats);

        Ok(Self {
            voice_command_sender,
            monitor_command_sender,
            _input_stream: input_stream,
            _voice_stream: voice_stream,
            _monitor_stream: monitor_stream,
            stats_thread_running,
        })
    }

    fn play_clip(
        &self,
        clip: Arc<AudioClip>,
        volume: f32,
        monitor_clip_playback: bool,
    ) -> Result<(), String> {
        let volume = volume.max(0.01);
        self.voice_command_sender
            .send(AudioCommand::PlayClip {
                clip: Arc::clone(&clip),
                volume,
            })
            .map_err(|err| err.to_string())?;
        if monitor_clip_playback {
            self.monitor_command_sender
                .send(AudioCommand::PlayClip { clip, volume })
                .map_err(|err| err.to_string())?;
        }

        Ok(())
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.stats_thread_running.store(false, Ordering::Relaxed);
    }
}

fn find_input_device(
    host: &cpal::Host,
    selected_name: Option<String>,
) -> Result<Option<cpal::Device>, cpal::DevicesError> {
    let Some(selected_name) = selected_name else {
        return Ok(None);
    };

    Ok(host.input_devices()?.find(|device| {
        device
            .description()
            .is_ok_and(|description| description.name() == selected_name)
    }))
}

fn find_monitor_output_device(
    host: &cpal::Host,
    selected_name: Option<String>,
) -> Result<Option<cpal::Device>, cpal::DevicesError> {
    let Some(selected_name) = selected_name else {
        return Ok(None);
    };

    Ok(host.output_devices()?.find(|device| {
        !is_vb_cable_device(device)
            && device
                .description()
                .is_ok_and(|description| description.name() == selected_name)
    }))
}

fn find_vb_cable_playback_device(
    host: &cpal::Host,
) -> Result<Option<cpal::Device>, cpal::DevicesError> {
    Ok(host.output_devices()?.find(|device| {
        device
            .description()
            .is_ok_and(|description| is_vb_cable_playback_name(description.name()))
    }))
}

fn first_non_vb_cable_output_device(
    host: &cpal::Host,
) -> Result<Option<cpal::Device>, cpal::DevicesError> {
    Ok(host
        .output_devices()?
        .find(|device| !is_vb_cable_device(device)))
}

fn vb_cable_status(host: &cpal::Host) -> Result<VbCableStatus, String> {
    let playback_device = find_vb_cable_playback_device(host)
        .map_err(|err| err.to_string())?
        .and_then(|device| output_device_info(&device).ok());

    Ok(VbCableStatus {
        installed: playback_device.is_some(),
        playback_device,
        voice_chat_input_name: "CABLE Output (VB-Audio Virtual Cable)".into(),
    })
}

fn is_vb_cable_playback_name(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    normalized.contains("cable input")
}

fn is_vb_cable_device(device: &cpal::Device) -> bool {
    device
        .description()
        .is_ok_and(|description| is_vb_cable_playback_name(description.name()))
}

fn prefill_silence(producer: &mut impl Producer<Item = f32>, frames: usize) {
    for _ in 0..frames {
        let _ = producer.try_push(0.0);
    }
}

fn start_stats_logger(stats: AudioStats) -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let thread_running = Arc::clone(&running);

    thread::spawn(move || {
        while thread_running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(2));

            let dropped = stats.dropped_input_frames.swap(0, Ordering::Relaxed);
            let missing = stats.missing_output_frames.swap(0, Ordering::Relaxed);

            if dropped > 0 {
                stats
                    .log
                    .push(format!("$ warning dropped_input_frames={dropped}"));
                eprintln!("Dropped {dropped} input frames. Try increasing LATENCY_MS.");
            }

            if missing > 0 {
                stats
                    .log
                    .push(format!("$ warning missing_output_frames={missing}"));
                eprintln!("Output needed {missing} missing frames. Try increasing LATENCY_MS.");
            }
        }
    });

    running
}

fn is_supported_clip_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "mp3" | "wav"))
}

fn clip_record_from_path(path: &Path) -> Result<ClipRecord, String> {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or("Clip file has no valid id")?
        .to_string();
    let id = clip_id_from_stem(&stem);
    let format = path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or("Clip file has no extension")?
        .to_ascii_lowercase();
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .ok_or("Clip file has no valid file name")?
        .to_string();

    Ok(ClipRecord {
        name: display_name_from_stem(&stem),
        id,
        file_name,
        format,
        path: path.display().to_string(),
    })
}

fn clip_id_from_stem(stem: &str) -> String {
    stem.split_once("__").map_or(stem, |(id, _)| id).to_string()
}

fn create_clip_id(name: &str) -> String {
    let slug = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    let slug = if slug.is_empty() { "clip".into() } else { slug };
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    format!("{slug}-{timestamp}")
}

fn display_name_from_id(id: &str) -> String {
    let without_timestamp = id.rsplit_once('-').map_or(id, |(name, _)| name);
    without_timestamp
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_name_from_stem(stem: &str) -> String {
    if let Some((_, display_name)) = stem.split_once("__") {
        return display_name.replace('-', " ");
    }

    display_name_from_id(stem)
}
