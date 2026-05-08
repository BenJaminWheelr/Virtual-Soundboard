use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;

fn main() {
    let host = cpal::default_host();
    // Use .id() to get the printable name (e.g., Wasapi or CoreAudio)
    println!("Default host: {}", host.id());

    let device = host.default_output_device().expect("Failed to get default output device");
    println!("Default output device: {}", device.description().expect("Failed to get output device information").name());

    let suppported_config = device.default_output_config().expect("Failed to get default output config");
    println!("Default output config: {:?}", suppported_config);

    match suppported_config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, suppported_config),
        _ => panic!("Unsupported sample format")
    }
    // println!("=== INPUT DEVICES ===");
    // for (i, device) in host.input_devices().unwrap().enumerate() {
    //     println!(
    //         "[{}] {} - Configs: {:?}\n\n\n\n\n\n",
    //         i,
    //         device.description().expect("Failed to get input device information").name(),
    //         device.default_input_config().expect("Failed to get default config"),
    //     );
    // }

    // println!("\n=== OUTPUT DEVICES ===");
    // for (i, device) in host.output_devices().unwrap().enumerate() {
    //     println!(
    //         "[{}] {} - Configs: \n\n\n\n\n\n",
    //         i,
    //         device.description().expect("Failed to get output device information").name(),
    //         // device.supported_output_configs().expect("Failed to get output device configs").collect::<Vec<_>>()
    //     );
    // }
}

fn run<T>(device: &cpal::Device, supported_config: cpal::SupportedStreamConfig)
where
    T: Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let config: cpal::StreamConfig = supported_config.into();

    let sample_rate = config.sample_rate as f32;
    let mut sample_clock = 0f32;

    // Spawn the high priority thread
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                // 1. Set the pitch (440.0Hz is a standard 'A' note)
                let freq = 330.0f32;
                
                // 2. Figure out if we are in the "High" or "Low" part of the wave
                // We use sine to get a smooth curve, then convert to a sample
                let half_period = sample_rate / freq / 2.0;
                let wave_value = if (sample_clock % (sample_rate / freq)) < half_period {
                    0.1f32 // Positive "pulse"
                } else {
                    -0.1f32 // Negative "pulse"
                };

                // 3. Convert that f32 math into whatever type T is (f32, i16, etc.)
                // This is where the Sample trait's "from_sample" power is used!
                *sample = T::from_sample::<f32>(wave_value);                
                sample_clock = (sample_clock + 1.0) % sample_rate;
            }
        },
        |err| eprintln!("Error in output stream: {}", err),
        None
    ).unwrap();

    stream.play().unwrap();

    println!("Stream is playing... Press Enter to stop.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}