use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

const DEFAULT_NOISE_GATE_THRESHOLD: f32 = 0.03;
const DEFAULT_HIGH_PASS_CUTOFF_HZ: f32 = 80.0;
const DEFAULT_LOW_PASS_CUTOFF_HZ: f32 = 12_000.0;
const DEFAULT_SATURATION_DRIVE: f32 = 1.5;

const NOISE_GATE_THRESHOLD_RANGE: std::ops::RangeInclusive<f32> = 0.0..=0.25;
const HIGH_PASS_CUTOFF_RANGE: std::ops::RangeInclusive<f32> = 20.0..=2_000.0;
const LOW_PASS_CUTOFF_RANGE: std::ops::RangeInclusive<f32> = 1_000.0..=20_000.0;
const SATURATION_DRIVE_RANGE: std::ops::RangeInclusive<f32> = 1.0..=8.0;

#[derive(Clone)]
pub struct SharedMicEffects {
    noise_gate_enabled: Arc<AtomicBool>,
    noise_gate_threshold: Arc<AtomicU32>,
    high_pass_enabled: Arc<AtomicBool>,
    high_pass_cutoff_hz: Arc<AtomicU32>,
    low_pass_enabled: Arc<AtomicBool>,
    low_pass_cutoff_hz: Arc<AtomicU32>,
    saturation_enabled: Arc<AtomicBool>,
    saturation_drive: Arc<AtomicU32>,
}

impl SharedMicEffects {
    pub fn new() -> Self {
        let effects = Self {
            noise_gate_enabled: Arc::new(AtomicBool::new(false)),
            noise_gate_threshold: Arc::new(AtomicU32::new(DEFAULT_NOISE_GATE_THRESHOLD.to_bits())),
            high_pass_enabled: Arc::new(AtomicBool::new(false)),
            high_pass_cutoff_hz: Arc::new(AtomicU32::new(DEFAULT_HIGH_PASS_CUTOFF_HZ.to_bits())),
            low_pass_enabled: Arc::new(AtomicBool::new(false)),
            low_pass_cutoff_hz: Arc::new(AtomicU32::new(DEFAULT_LOW_PASS_CUTOFF_HZ.to_bits())),
            saturation_enabled: Arc::new(AtomicBool::new(false)),
            saturation_drive: Arc::new(AtomicU32::new(DEFAULT_SATURATION_DRIVE.to_bits())),
        };
        effects.set_config(MicEffectsConfig::default());
        effects
    }

    pub fn config(&self) -> MicEffectsConfig {
        MicEffectsConfig {
            noise_gate: NoiseGateConfig {
                enabled: self.noise_gate_enabled.load(Ordering::Relaxed),
                threshold: load_f32(&self.noise_gate_threshold),
            },
            high_pass: FilterConfig {
                enabled: self.high_pass_enabled.load(Ordering::Relaxed),
                cutoff_hz: load_f32(&self.high_pass_cutoff_hz),
            },
            low_pass: FilterConfig {
                enabled: self.low_pass_enabled.load(Ordering::Relaxed),
                cutoff_hz: load_f32(&self.low_pass_cutoff_hz),
            },
            saturation: SaturationConfig {
                enabled: self.saturation_enabled.load(Ordering::Relaxed),
                drive: load_f32(&self.saturation_drive),
            },
        }
    }

    pub fn set_config(&self, config: MicEffectsConfig) {
        self.noise_gate_enabled
            .store(config.noise_gate.enabled, Ordering::Relaxed);
        store_f32(
            &self.noise_gate_threshold,
            clamp_to_range(config.noise_gate.threshold, &NOISE_GATE_THRESHOLD_RANGE),
        );
        self.high_pass_enabled
            .store(config.high_pass.enabled, Ordering::Relaxed);
        store_f32(
            &self.high_pass_cutoff_hz,
            clamp_to_range(config.high_pass.cutoff_hz, &HIGH_PASS_CUTOFF_RANGE),
        );
        self.low_pass_enabled
            .store(config.low_pass.enabled, Ordering::Relaxed);
        store_f32(
            &self.low_pass_cutoff_hz,
            clamp_to_range(config.low_pass.cutoff_hz, &LOW_PASS_CUTOFF_RANGE),
        );
        self.saturation_enabled
            .store(config.saturation.enabled, Ordering::Relaxed);
        store_f32(
            &self.saturation_drive,
            clamp_to_range(config.saturation.drive, &SATURATION_DRIVE_RANGE),
        );
    }

    pub fn snapshot(&self) -> MicEffectsSnapshot {
        MicEffectsSnapshot {
            noise_gate_enabled: self.noise_gate_enabled.load(Ordering::Relaxed),
            noise_gate_threshold: load_f32(&self.noise_gate_threshold),
            high_pass_enabled: self.high_pass_enabled.load(Ordering::Relaxed),
            high_pass_cutoff_hz: load_f32(&self.high_pass_cutoff_hz),
            low_pass_enabled: self.low_pass_enabled.load(Ordering::Relaxed),
            low_pass_cutoff_hz: load_f32(&self.low_pass_cutoff_hz),
            saturation_enabled: self.saturation_enabled.load(Ordering::Relaxed),
            saturation_drive: load_f32(&self.saturation_drive),
        }
    }
}

#[derive(Clone, Copy)]
pub struct MicEffectsSnapshot {
    noise_gate_enabled: bool,
    noise_gate_threshold: f32,
    high_pass_enabled: bool,
    high_pass_cutoff_hz: f32,
    low_pass_enabled: bool,
    low_pass_cutoff_hz: f32,
    saturation_enabled: bool,
    saturation_drive: f32,
}

pub struct MicEffectsProcessor {
    effects: SharedMicEffects,
    sample_rate: f32,
    gate_envelope: f32,
    high_pass_previous_input: f32,
    high_pass_previous_output: f32,
    low_pass_previous_output: f32,
}

impl MicEffectsProcessor {
    pub fn new(effects: SharedMicEffects, sample_rate: u32) -> Self {
        Self {
            effects,
            sample_rate: sample_rate as f32,
            gate_envelope: 0.0,
            high_pass_previous_input: 0.0,
            high_pass_previous_output: 0.0,
            low_pass_previous_output: 0.0,
        }
    }

    pub fn process_sample(&mut self, mut sample: f32) -> f32 {
        let snapshot = self.effects.snapshot();

        if snapshot.noise_gate_enabled {
            sample = self.apply_noise_gate(sample, snapshot.noise_gate_threshold);
        }
        if snapshot.high_pass_enabled {
            sample = self.apply_high_pass(sample, snapshot.high_pass_cutoff_hz);
        }
        if snapshot.low_pass_enabled {
            sample = self.apply_low_pass(sample, snapshot.low_pass_cutoff_hz);
        }
        if snapshot.saturation_enabled {
            sample = apply_saturation(sample, snapshot.saturation_drive);
        }

        sample.clamp(-1.0, 1.0)
    }

    fn apply_noise_gate(&mut self, sample: f32, threshold: f32) -> f32 {
        let target = if sample.abs() >= threshold { 1.0 } else { 0.0 };
        let coefficient = if target > self.gate_envelope {
            0.08
        } else {
            0.003
        };
        self.gate_envelope += (target - self.gate_envelope) * coefficient;
        sample * self.gate_envelope
    }

    fn apply_high_pass(&mut self, sample: f32, cutoff_hz: f32) -> f32 {
        let cutoff_hz = cutoff_hz.clamp(20.0, self.sample_rate * 0.45);
        let rc = 1.0 / (2.0 * PI * cutoff_hz);
        let dt = 1.0 / self.sample_rate;
        let alpha = rc / (rc + dt);
        let output =
            alpha * (self.high_pass_previous_output + sample - self.high_pass_previous_input);
        self.high_pass_previous_input = sample;
        self.high_pass_previous_output = output;
        output
    }

    fn apply_low_pass(&mut self, sample: f32, cutoff_hz: f32) -> f32 {
        let cutoff_hz = cutoff_hz.clamp(20.0, self.sample_rate * 0.45);
        let rc = 1.0 / (2.0 * PI * cutoff_hz);
        let dt = 1.0 / self.sample_rate;
        let alpha = dt / (rc + dt);
        self.low_pass_previous_output += alpha * (sample - self.low_pass_previous_output);
        self.low_pass_previous_output
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MicEffectsConfig {
    pub noise_gate: NoiseGateConfig,
    pub high_pass: FilterConfig,
    pub low_pass: FilterConfig,
    pub saturation: SaturationConfig,
}

impl Default for MicEffectsConfig {
    fn default() -> Self {
        Self {
            noise_gate: NoiseGateConfig {
                enabled: false,
                threshold: DEFAULT_NOISE_GATE_THRESHOLD,
            },
            high_pass: FilterConfig {
                enabled: false,
                cutoff_hz: DEFAULT_HIGH_PASS_CUTOFF_HZ,
            },
            low_pass: FilterConfig {
                enabled: false,
                cutoff_hz: DEFAULT_LOW_PASS_CUTOFF_HZ,
            },
            saturation: SaturationConfig {
                enabled: false,
                drive: DEFAULT_SATURATION_DRIVE,
            },
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NoiseGateConfig {
    pub enabled: bool,
    pub threshold: f32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct FilterConfig {
    pub enabled: bool,
    pub cutoff_hz: f32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SaturationConfig {
    pub enabled: bool,
    pub drive: f32,
}

fn apply_saturation(sample: f32, drive: f32) -> f32 {
    let drive = drive.max(1.0);
    (sample * drive).tanh() / drive.tanh()
}

fn load_f32(value: &AtomicU32) -> f32 {
    f32::from_bits(value.load(Ordering::Relaxed))
}

fn store_f32(value: &AtomicU32, next_value: f32) {
    value.store(next_value.to_bits(), Ordering::Relaxed);
}

fn clamp_to_range(value: f32, range: &std::ops::RangeInclusive<f32>) -> f32 {
    value.clamp(*range.start(), *range.end())
}
