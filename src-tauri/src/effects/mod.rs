mod simple;

use serde::{Deserialize, Serialize};
use simple::{SimpleEffectsProcessor, apply_saturation};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

const DEFAULT_HIGH_PASS_CUTOFF_HZ: f32 = 80.0;
const DEFAULT_LOW_PASS_CUTOFF_HZ: f32 = 12_000.0;
const DEFAULT_SATURATION_DRIVE: f32 = 1.5;

const HIGH_PASS_CUTOFF_RANGE: std::ops::RangeInclusive<f32> = 20.0..=2_000.0;
const LOW_PASS_CUTOFF_RANGE: std::ops::RangeInclusive<f32> = 1_000.0..=20_000.0;
const SATURATION_DRIVE_RANGE: std::ops::RangeInclusive<f32> = 1.0..=8.0;

#[derive(Clone)]
pub struct SharedMicEffects {
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
    pub high_pass_enabled: bool,
    pub high_pass_cutoff_hz: f32,
    pub low_pass_enabled: bool,
    pub low_pass_cutoff_hz: f32,
    pub saturation_enabled: bool,
    pub saturation_drive: f32,
}

pub struct MicEffectsProcessor {
    effects: SharedMicEffects,
    simple: SimpleEffectsProcessor,
}

impl MicEffectsProcessor {
    pub fn new(effects: SharedMicEffects, sample_rate: u32) -> Self {
        Self {
            effects,
            simple: SimpleEffectsProcessor::new(sample_rate),
        }
    }

    pub fn process_sample(&mut self, mut sample: f32) -> f32 {
        let snapshot = self.effects.snapshot();

        if snapshot.high_pass_enabled {
            sample = self
                .simple
                .apply_high_pass(sample, snapshot.high_pass_cutoff_hz);
        }
        if snapshot.low_pass_enabled {
            sample = self
                .simple
                .apply_low_pass(sample, snapshot.low_pass_cutoff_hz);
        }
        if snapshot.saturation_enabled {
            sample = apply_saturation(sample, snapshot.saturation_drive);
        }

        sample.clamp(-1.0, 1.0)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MicEffectsConfig {
    #[serde(default = "default_high_pass_config")]
    pub high_pass: FilterConfig,
    #[serde(default = "default_low_pass_config")]
    pub low_pass: FilterConfig,
    #[serde(default = "default_saturation_config")]
    pub saturation: SaturationConfig,
}

impl Default for MicEffectsConfig {
    fn default() -> Self {
        Self {
            high_pass: default_high_pass_config(),
            low_pass: default_low_pass_config(),
            saturation: default_saturation_config(),
        }
    }
}

fn default_high_pass_config() -> FilterConfig {
    FilterConfig {
        enabled: false,
        cutoff_hz: DEFAULT_HIGH_PASS_CUTOFF_HZ,
    }
}

fn default_low_pass_config() -> FilterConfig {
    FilterConfig {
        enabled: false,
        cutoff_hz: DEFAULT_LOW_PASS_CUTOFF_HZ,
    }
}

fn default_saturation_config() -> SaturationConfig {
    SaturationConfig {
        enabled: false,
        drive: DEFAULT_SATURATION_DRIVE,
    }
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

fn load_f32(value: &AtomicU32) -> f32 {
    f32::from_bits(value.load(Ordering::Relaxed))
}

fn store_f32(value: &AtomicU32, next_value: f32) {
    value.store(next_value.to_bits(), Ordering::Relaxed);
}

fn clamp_to_range(value: f32, range: &std::ops::RangeInclusive<f32>) -> f32 {
    value.clamp(*range.start(), *range.end())
}
