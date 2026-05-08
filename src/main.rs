mod audio;
mod clip;

use audio::{build_input_stream, build_output_stream, AudioStats};
use clip::AudioClip;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{traits::{Producer, Split}, HeapRb};
use std::error::Error;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

const LATENCY_MS: f32 = 80.0;
const CLIP_PATH: &str = "D:\\Projects\\virtual_soundboard\\sampleAudioClip.mp3";

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

    println!("Input config: {input_config:?}");
    println!("Output config: {output_config:?}");

    let input_sample_rate = input_config.sample_rate as f32;
    let output_sample_rate = output_config.sample_rate as f32;
    let input_channels = input_config.channels as usize;
    let output_channels = output_config.channels as usize;

    let clip = AudioClip::load(CLIP_PATH)?;
    println!(
        "Loaded audio clip: {} frames at {} Hz",
        clip.samples.len(),
        clip.sample_rate
    );

    let latency_frames = ((LATENCY_MS / 1_000.0) * input_sample_rate).round() as usize;
    let rb = HeapRb::<f32>::new((latency_frames * 4).max(1024));
    let (mut producer, consumer) = rb.split();
    prefill_silence(&mut producer, latency_frames);

    let stats = AudioStats::new();

    let input_stream = build_input_stream(
        &input_device,
        &input_config,
        input_supported_config.sample_format(),
        input_channels,
        producer,
        stats.dropped_input_frames.clone(),
    )?;

    let output_stream = build_output_stream(
        &output_device,
        &output_config,
        output_supported_config.sample_format(),
        output_channels,
        input_sample_rate / output_sample_rate,
        clip,
        consumer,
        stats.missing_output_frames.clone(),
    )?;

    input_stream.play()?;
    output_stream.play()?;

    start_stats_logger(stats);

    println!("Monitoring with about {LATENCY_MS} ms of buffering. Press Enter to stop.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}

fn prefill_silence(producer: &mut impl Producer<Item = f32>, frames: usize) {
    for _ in 0..frames {
        let _ = producer.try_push(0.0);
    }
}

fn start_stats_logger(stats: AudioStats) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(2));

        let dropped = stats.dropped_input_frames.swap(0, Ordering::Relaxed);
        let missing = stats.missing_output_frames.swap(0, Ordering::Relaxed);

        if dropped > 0 {
            eprintln!("Dropped {dropped} input frames. Try increasing LATENCY_MS.");
        }

        if missing > 0 {
            eprintln!("Output needed {missing} missing frames. Try increasing LATENCY_MS.");
        }
    });
}
