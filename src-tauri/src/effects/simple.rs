use std::f32::consts::PI;

pub struct SimpleEffectsProcessor {
    sample_rate: f32,
    high_pass_previous_input: f32,
    high_pass_previous_output: f32,
    low_pass_previous_output: f32,
}

impl SimpleEffectsProcessor {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate as f32,
            high_pass_previous_input: 0.0,
            high_pass_previous_output: 0.0,
            low_pass_previous_output: 0.0,
        }
    }

    pub fn apply_high_pass(&mut self, sample: f32, cutoff_hz: f32) -> f32 {
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

    pub fn apply_low_pass(&mut self, sample: f32, cutoff_hz: f32) -> f32 {
        let cutoff_hz = cutoff_hz.clamp(20.0, self.sample_rate * 0.45);
        let rc = 1.0 / (2.0 * PI * cutoff_hz);
        let dt = 1.0 / self.sample_rate;
        let alpha = dt / (rc + dt);
        self.low_pass_previous_output += alpha * (sample - self.low_pass_previous_output);
        self.low_pass_previous_output
    }
}

pub fn apply_saturation(sample: f32, drive: f32) -> f32 {
    let drive = drive.max(1.0);
    (sample * drive).tanh() / drive.tanh()
}
