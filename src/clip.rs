use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct AudioClip {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl AudioClip {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        match path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref()
        {
            Some("wav") => load_wav_mono(path),
            Some("mp3") => load_mp3_mono(path),
            Some(extension) => Err(format!("Unsupported audio file extension: {extension}").into()),
            None => Err("Audio file path has no extension".into()),
        }
    }
}

pub struct AudioClipPlayer {
    clip: AudioClip,
    position: f32,
    step: f32,
}

impl AudioClipPlayer {
    pub fn new(clip: AudioClip, output_sample_rate: u32) -> Self {
        let step = clip.sample_rate as f32 / output_sample_rate as f32;

        Self {
            clip,
            position: 0.0,
            step,
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let sample = sample_interpolated(&self.clip.samples, self.position);
        self.position += self.step;
        sample
    }
}

fn load_wav_mono(path: &Path) -> Result<AudioClip, Box<dyn Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|sample| sample.map(|sample| sample.clamp(-1.0, 1.0)))
            .collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let max_amplitude = 2_f32.powi(spec.bits_per_sample as i32 - 1);
            reader
                .samples::<i32>()
                .map(|sample| {
                    sample.map(|sample| (sample as f32 / max_amplitude).clamp(-1.0, 1.0))
                })
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    let mono_samples = samples
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect();

    Ok(AudioClip {
        samples: mono_samples,
        sample_rate: spec.sample_rate,
    })
}

fn load_mp3_mono(path: &Path) -> Result<AudioClip, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut decoder = minimp3::Decoder::new(file);
    let mut samples = Vec::new();
    let mut sample_rate = None;

    loop {
        match decoder.next_frame() {
            Ok(frame) => {
                sample_rate.get_or_insert(frame.sample_rate as u32);

                for frame_samples in frame.data.chunks(frame.channels) {
                    let mono = frame_samples
                        .iter()
                        .map(|sample| *sample as f32 / i16::MAX as f32)
                        .sum::<f32>()
                        / frame_samples.len() as f32;
                    samples.push(mono.clamp(-1.0, 1.0));
                }
            }
            Err(minimp3::Error::Eof) => break,
            Err(err) => return Err(Box::new(err)),
        }
    }

    Ok(AudioClip {
        samples,
        sample_rate: sample_rate.ok_or("MP3 contained no audio frames")?,
    })
}

fn sample_interpolated(samples: &[f32], position: f32) -> f32 {
    let index = position.floor() as usize;
    if index + 1 >= samples.len() {
        return 0.0;
    }

    let fraction = position - index as f32;
    samples[index] + (samples[index + 1] - samples[index]) * fraction
}
