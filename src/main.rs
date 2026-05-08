use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
use std::error::Error;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

const LATENCY_MS: f32 = 80.0;

fn main() -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();
    println!("Default host: {}", host.id());

    let input_device = host
        .default_input_device()
        .expect("Failed to get default input device");
    println!(
        "Default input device: {}",
        input_device.description()?.name()
    );

    let output_device = host
        .default_output_device()
        .expect("Failed to get default output device");
    println!(
        "Default output device: {}",
        output_device.description()?.name()
    );

    let input_supported_config = input_device.default_input_config()?;
    let output_supported_config = output_device.default_output_config()?;
    let input_config: cpal::StreamConfig = input_supported_config.clone().into();
    let output_config: cpal::StreamConfig = output_supported_config.clone().into();

    println!("Input config: {input_supported_config:?}");
    println!("Output config: {output_supported_config:?}");

    let input_sample_rate = input_config.sample_rate as f32;
    let output_sample_rate = output_config.sample_rate as f32;
    let input_channels = input_config.channels as usize;
    let output_channels = output_config.channels as usize;

    let latency_frames = ((LATENCY_MS / 1_000.0) * input_sample_rate).round() as usize;
    let ring_capacity = latency_frames * 4;
    let rb = HeapRb::<f32>::new(ring_capacity.max(1024));
    let (mut producer, consumer) = rb.split();

    for _ in 0..latency_frames {
        let _ = producer.try_push(0.0);
    }

    let dropped_input_frames = Arc::new(AtomicUsize::new(0));
    let missing_output_frames = Arc::new(AtomicUsize::new(0));

    let input_stream = build_input_stream(
        &input_device,
        &input_config,
        input_supported_config.sample_format(),
        input_channels,
        producer,
        Arc::clone(&dropped_input_frames),
    )?;

    let output_stream = build_output_stream(
        &output_device,
        &output_config,
        output_supported_config.sample_format(),
        output_channels,
        input_sample_rate / output_sample_rate,
        consumer,
        Arc::clone(&missing_output_frames),
    )?;

    input_stream.play()?;
    output_stream.play()?;

    println!(
        "Monitoring with about {LATENCY_MS} ms of buffering. Press Enter to stop."
    );

    let stats_dropped = Arc::clone(&dropped_input_frames);
    let stats_missing = Arc::clone(&missing_output_frames);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(2));

        let dropped = stats_dropped.swap(0, Ordering::Relaxed);
        let missing = stats_missing.swap(0, Ordering::Relaxed);

        if dropped > 0 {
            eprintln!("Dropped {dropped} input frames. Try increasing LATENCY_MS.");
        }

        if missing > 0 {
            eprintln!("Output needed {missing} missing frames. Try increasing LATENCY_MS.");
        }
    });

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}

fn build_input_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: SampleFormat,
    input_channels: usize,
    producer: impl Producer<Item = f32> + Send + 'static,
    dropped_frames: Arc<AtomicUsize>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    match sample_format {
        SampleFormat::I8 => {
            build_typed_input_stream::<i8>(device, config, input_channels, producer, dropped_frames)
        }
        SampleFormat::I16 => {
            build_typed_input_stream::<i16>(device, config, input_channels, producer, dropped_frames)
        }
        SampleFormat::I24 => build_typed_input_stream::<cpal::I24>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
        ),
        SampleFormat::I32 => {
            build_typed_input_stream::<i32>(device, config, input_channels, producer, dropped_frames)
        }
        SampleFormat::I64 => {
            build_typed_input_stream::<i64>(device, config, input_channels, producer, dropped_frames)
        }
        SampleFormat::U8 => {
            build_typed_input_stream::<u8>(device, config, input_channels, producer, dropped_frames)
        }
        SampleFormat::U16 => {
            build_typed_input_stream::<u16>(device, config, input_channels, producer, dropped_frames)
        }
        SampleFormat::U24 => build_typed_input_stream::<cpal::U24>(
            device,
            config,
            input_channels,
            producer,
            dropped_frames,
        ),
        SampleFormat::U32 => {
            build_typed_input_stream::<u32>(device, config, input_channels, producer, dropped_frames)
        }
        SampleFormat::U64 => {
            build_typed_input_stream::<u64>(device, config, input_channels, producer, dropped_frames)
        }
        SampleFormat::F32 => {
            build_typed_input_stream::<f32>(device, config, input_channels, producer, dropped_frames)
        }
        SampleFormat::F64 => {
            build_typed_input_stream::<f64>(device, config, input_channels, producer, dropped_frames)
        }
        format => panic!("Unsupported input sample format: {format:?}"),
    }
}

fn build_typed_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    input_channels: usize,
    mut producer: impl Producer<Item = f32> + Send + 'static,
    dropped_frames: Arc<AtomicUsize>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: Sample + SizedSample,
    f32: FromSample<T>,
{
    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            for frame in data.chunks(input_channels) {
                let mono = frame
                    .iter()
                    .map(|sample| sample.to_sample::<f32>())
                    .sum::<f32>()
                    / frame.len() as f32;

                if producer.try_push(mono).is_err() {
                    dropped_frames.fetch_add(1, Ordering::Relaxed);
                }
            }
        },
        err_fn,
        None,
    )
}

fn build_output_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: SampleFormat,
    output_channels: usize,
    resample_step: f32,
    consumer: impl Consumer<Item = f32> + Send + 'static,
    missing_frames: Arc<AtomicUsize>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    match sample_format {
        SampleFormat::I8 => build_typed_output_stream::<i8>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        SampleFormat::I16 => build_typed_output_stream::<i16>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        SampleFormat::I24 => build_typed_output_stream::<cpal::I24>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        SampleFormat::I32 => build_typed_output_stream::<i32>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        SampleFormat::I64 => build_typed_output_stream::<i64>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        SampleFormat::U8 => build_typed_output_stream::<u8>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        SampleFormat::U16 => build_typed_output_stream::<u16>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        SampleFormat::U24 => build_typed_output_stream::<cpal::U24>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        SampleFormat::U32 => build_typed_output_stream::<u32>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        SampleFormat::U64 => build_typed_output_stream::<u64>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        SampleFormat::F32 => build_typed_output_stream::<f32>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        SampleFormat::F64 => build_typed_output_stream::<f64>(
            device,
            config,
            output_channels,
            resample_step,
            consumer,
            missing_frames,
        ),
        format => panic!("Unsupported output sample format: {format:?}"),
    }
}

fn build_typed_output_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    output_channels: usize,
    resample_step: f32,
    mut consumer: impl Consumer<Item = f32> + Send + 'static,
    missing_frames: Arc<AtomicUsize>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: FromSample<f32> + SizedSample,
{
    let mut previous = 0.0;
    let mut next = 0.0;
    let mut position = 1.0;

    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
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

                let mono = previous + (next - previous) * position;
                position += resample_step;

                for sample in frame {
                    *sample = T::from_sample(mono);
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
