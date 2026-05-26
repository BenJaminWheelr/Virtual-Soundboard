use crate::clip::{AudioClip, AudioClipPlayer};
use crate::effects::{MicEffectsProcessor, SharedMicEffects};
use cpal::traits::DeviceTrait;
use cpal::{FromSample, Sample, SampleFormat, SizedSample};
use ringbuf::traits::{Consumer, Producer};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc::Receiver,
};

pub enum AudioCommand {
    PlayClip { clip: Arc<AudioClip>, volume: f32 },
}

pub struct AudioStats {
    pub dropped_input_frames: Arc<AtomicUsize>,
    pub missing_output_frames: Arc<AtomicUsize>,
}

impl AudioStats {
    pub fn new() -> Self {
        Self {
            dropped_input_frames: Arc::new(AtomicUsize::new(0)),
            missing_output_frames: Arc::new(AtomicUsize::new(0)),
        }
    }
}

pub fn build_input_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: SampleFormat,
    input_channels: usize,
    producer: impl Producer<Item = f32> + Send + 'static,
    dropped_frames: Arc<AtomicUsize>,
    mic_effects: SharedMicEffects,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    match sample_format {
        SampleFormat::I8 => build_typed_input_stream::<i8>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        SampleFormat::I16 => build_typed_input_stream::<i16>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        SampleFormat::I24 => build_typed_input_stream::<cpal::I24>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        SampleFormat::I32 => build_typed_input_stream::<i32>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        SampleFormat::I64 => build_typed_input_stream::<i64>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        SampleFormat::U8 => build_typed_input_stream::<u8>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        SampleFormat::U16 => build_typed_input_stream::<u16>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        SampleFormat::U24 => build_typed_input_stream::<cpal::U24>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        SampleFormat::U32 => build_typed_input_stream::<u32>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        SampleFormat::U64 => build_typed_input_stream::<u64>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        SampleFormat::F32 => build_typed_input_stream::<f32>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        SampleFormat::F64 => build_typed_input_stream::<f64>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
            mic_effects,
        ),
        format => panic!("Unsupported input sample format: {format:?}"),
    }
}

fn build_typed_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    input_channels: usize,
    mut producer: impl Producer<Item = f32> + Send + 'static,
    dropped_frames: Arc<AtomicUsize>,
    mic_effects: SharedMicEffects,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: Sample + SizedSample,
    f32: FromSample<T>,
{
    let mut processor = MicEffectsProcessor::new(mic_effects, config.sample_rate);

    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            for frame in data.chunks(input_channels) {
                let mono = frame
                    .iter()
                    .map(|sample| sample.to_sample::<f32>())
                    .sum::<f32>()
                    / frame.len() as f32;

                let processed = processor.process_sample(mono);

                if producer.try_push(processed).is_err() {
                    dropped_frames.fetch_add(1, Ordering::Relaxed);
                }
            }
        },
        err_fn,
        None,
    )
}

pub fn build_output_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: SampleFormat,
    output_channels: usize,
    mic_resample_step: f32,
    command_receiver: Receiver<AudioCommand>,
    consumer: impl Consumer<Item = f32> + Send + 'static,
    missing_frames: Arc<AtomicUsize>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    match sample_format {
        SampleFormat::I8 => build_typed_output_stream::<i8>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        SampleFormat::I16 => build_typed_output_stream::<i16>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        SampleFormat::I24 => build_typed_output_stream::<cpal::I24>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        SampleFormat::I32 => build_typed_output_stream::<i32>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        SampleFormat::I64 => build_typed_output_stream::<i64>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        SampleFormat::U8 => build_typed_output_stream::<u8>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        SampleFormat::U16 => build_typed_output_stream::<u16>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        SampleFormat::U24 => build_typed_output_stream::<cpal::U24>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        SampleFormat::U32 => build_typed_output_stream::<u32>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        SampleFormat::U64 => build_typed_output_stream::<u64>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        SampleFormat::F32 => build_typed_output_stream::<f32>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        SampleFormat::F64 => build_typed_output_stream::<f64>(
            device,
            config,
            output_channels,
            mic_resample_step,
            command_receiver,
            consumer,
            missing_frames,
        ),
        format => panic!("Unsupported output sample format: {format:?}"),
    }
}

pub fn build_clip_monitor_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: SampleFormat,
    output_channels: usize,
    command_receiver: Receiver<AudioCommand>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    match sample_format {
        SampleFormat::I8 => {
            build_typed_clip_monitor_stream::<i8>(device, config, output_channels, command_receiver)
        }
        SampleFormat::I16 => build_typed_clip_monitor_stream::<i16>(
            device,
            config,
            output_channels,
            command_receiver,
        ),
        SampleFormat::I24 => build_typed_clip_monitor_stream::<cpal::I24>(
            device,
            config,
            output_channels,
            command_receiver,
        ),
        SampleFormat::I32 => build_typed_clip_monitor_stream::<i32>(
            device,
            config,
            output_channels,
            command_receiver,
        ),
        SampleFormat::I64 => build_typed_clip_monitor_stream::<i64>(
            device,
            config,
            output_channels,
            command_receiver,
        ),
        SampleFormat::U8 => {
            build_typed_clip_monitor_stream::<u8>(device, config, output_channels, command_receiver)
        }
        SampleFormat::U16 => build_typed_clip_monitor_stream::<u16>(
            device,
            config,
            output_channels,
            command_receiver,
        ),
        SampleFormat::U24 => build_typed_clip_monitor_stream::<cpal::U24>(
            device,
            config,
            output_channels,
            command_receiver,
        ),
        SampleFormat::U32 => build_typed_clip_monitor_stream::<u32>(
            device,
            config,
            output_channels,
            command_receiver,
        ),
        SampleFormat::U64 => build_typed_clip_monitor_stream::<u64>(
            device,
            config,
            output_channels,
            command_receiver,
        ),
        SampleFormat::F32 => build_typed_clip_monitor_stream::<f32>(
            device,
            config,
            output_channels,
            command_receiver,
        ),
        SampleFormat::F64 => build_typed_clip_monitor_stream::<f64>(
            device,
            config,
            output_channels,
            command_receiver,
        ),
        format => panic!("Unsupported monitor sample format: {format:?}"),
    }
}

fn build_typed_clip_monitor_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    output_channels: usize,
    command_receiver: Receiver<AudioCommand>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: FromSample<f32> + SizedSample,
{
    let mut active_clips = Vec::<AudioClipPlayer>::new();
    let output_sample_rate = config.sample_rate;

    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            while let Ok(command) = command_receiver.try_recv() {
                match command {
                    AudioCommand::PlayClip { clip, volume } => {
                        active_clips.push(AudioClipPlayer::new(clip, output_sample_rate, volume));
                    }
                }
            }

            for frame in data.chunks_mut(output_channels) {
                let mut mixed = 0.0;
                active_clips.retain_mut(|clip| {
                    if let Some(sample) = clip.next_sample() {
                        mixed += sample;
                        true
                    } else {
                        false
                    }
                });
                mixed = mixed.clamp(-1.0, 1.0);

                for sample in frame {
                    *sample = T::from_sample(mixed);
                }
            }
        },
        err_fn,
        None,
    )
}

fn build_typed_output_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    output_channels: usize,
    mic_resample_step: f32,
    command_receiver: Receiver<AudioCommand>,
    mut consumer: impl Consumer<Item = f32> + Send + 'static,
    missing_frames: Arc<AtomicUsize>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: FromSample<f32> + SizedSample,
{
    let mut previous = 0.0;
    let mut next = 0.0;
    let mut position = 1.0;
    let mut active_clips = Vec::<AudioClipPlayer>::new();
    let output_sample_rate = config.sample_rate;

    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            while let Ok(command) = command_receiver.try_recv() {
                match command {
                    AudioCommand::PlayClip { clip, volume } => {
                        active_clips.push(AudioClipPlayer::new(clip, output_sample_rate, volume));
                    }
                }
            }

            for frame in data.chunks_mut(output_channels) {
                while position >= 1.0 {
                    previous = next;
                    match consumer.try_pop() {
                        Some(sample) => next = sample,
                        None => {
                            next = 0.0;
                            missing_frames.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    position -= 1.0;
                }

                let mic_sample = previous + (next - previous) * position;
                position += mic_resample_step;

                let mut mixed = mic_sample;
                active_clips.retain_mut(|clip| {
                    if let Some(sample) = clip.next_sample() {
                        mixed += sample;
                        true
                    } else {
                        false
                    }
                });
                mixed = mixed.clamp(-1.0, 1.0);

                for sample in frame {
                    *sample = T::from_sample(mixed);
                }
            }
        },
        err_fn,
        None,
    )
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("Audio stream error: {err}");
}
