# Audio Engine

The audio engine is the heart of the app. It is implemented mainly in:

- `src-tauri/src/soundboard.rs`
- `src-tauri/src/audio.rs`
- `src-tauri/src/clip.rs`
- `src-tauri/src/effects/`

The engine uses `cpal` for cross-platform audio device access.

## Routing Model

The app creates three streams when the audio engine starts:

1. Input stream: reads the physical microphone.
2. Voice output stream: writes mic plus clips to VB-Cable.
3. Monitor output stream: writes clips only to headphones/speakers.

The routing looks like this:

```text
physical mic
-> input stream
-> mic effects
-> ring buffer
-> voice output stream
-> VB-Cable
-> Discord/game
```

Clips are injected into the voice output stream:

```text
clip command
-> voice output stream
-> mixed with mic
-> VB-Cable
```

Clips are optionally injected into the monitor stream:

```text
clip command
-> monitor output stream
-> headphones/speakers
```

## AudioEngine

`AudioEngine` lives in `soundboard.rs`.

It owns:

- `voice_command_sender`
- `monitor_command_sender`
- `_input_stream`
- `_voice_stream`
- `_monitor_stream`
- `_stats_thread_running`

The stream fields are prefixed with `_` because the code does not call methods on them after startup. They are still important: keeping the stream objects alive keeps the streams running. Dropping them stops audio.

## Starting The Engine

`AudioEngine::start`:

1. Gets the default CPAL host.
2. Finds the selected input device, or falls back to the default input.
3. Finds the VB-Cable playback device.
4. Finds the selected monitor output, or falls back to a non-VB-Cable output.
5. Reads default stream configs from each device.
6. Creates a ring buffer for mic audio.
7. Creates command channels for clips.
8. Builds the input, voice output, and monitor streams.
9. Starts all streams with `.play()`.
10. Starts a lightweight stats logger thread.

## Device Selection

The frontend sends:

```ts
{
  input_device: selectedInput || null,
  monitor_output_device: selectedMonitorOutput || null
}
```

Rust deserializes that into:

```rust
pub struct DeviceSelection {
    pub input_device: Option<String>,
    pub monitor_output_device: Option<String>,
}
```

If an option is `None`, the backend tries a sensible default.

## VB-Cable Detection

VB-Cable exposes an output playback device commonly named:

```text
CABLE Input (VB-Audio Virtual Cable)
```

Discord/game uses the matching recording device:

```text
CABLE Output (VB-Audio Virtual Cable)
```

The app writes audio to `CABLE Input`. Other apps receive it from `CABLE Output`.

The backend identifies the playback device by checking whether the output device name contains:

```text
cable input
```

## Ring Buffer

The mic input stream and voice output stream run independently. They need a thread-safe way to pass mic samples from input to output.

The app uses a ring buffer from `ringbuf`:

```text
input stream producer -> ring buffer -> output stream consumer
```

The ring buffer is prefilled with silence based on `LATENCY_MS`. This gives the output stream a small cushion so it does not immediately starve.

## Input Stream

The input stream is built in `audio.rs` by `build_input_stream`.

CPAL devices can use different sample formats:

- `i8`, `i16`, `i24`, `i32`, `i64`
- `u8`, `u16`, `u24`, `u32`, `u64`
- `f32`, `f64`

`build_input_stream` matches the sample format and calls a typed helper.

Inside the input callback:

1. Read one frame from the microphone.
2. Average all input channels into mono.
3. Run the mono sample through `MicEffectsProcessor`.
4. Push the processed sample into the ring buffer.
5. If the buffer is full, increment `dropped_input_frames`.

The mic effects happen here, before mic audio reaches the voice output stream.

## Voice Output Stream

The voice output stream is built by `build_output_stream`.

Inside the output callback:

1. Drain any pending `AudioCommand::PlayClip` messages.
2. Pull/resample the next mic sample from the ring buffer.
3. Advance all active clip players.
4. Mix mic plus clips.
5. Clamp the mix to `[-1.0, 1.0]`.
6. Write the same mixed sample to each output channel.

This stream writes to VB-Cable.

## Monitor Output Stream

The monitor stream is built by `build_clip_monitor_stream`.

It is similar to the voice output stream, but it does not read the mic ring buffer. It only plays active clips.

That means:

- Discord/game hears mic plus clips.
- The user hears clips only.

## Clip Commands

Clip playback is command-based.

`Soundboard.play_clip` sends `AudioCommand::PlayClip` into the voice stream channel. If monitor playback is enabled, it also sends the same command into the monitor stream channel.

The output streams receive commands inside their audio callbacks with `try_recv`, which avoids blocking.

## Clip Decoding And Playback

`clip.rs` defines `AudioClip`.

When a clip is loaded:

- WAV files are decoded with `hound`.
- MP3 files are decoded with `minimp3`.
- Multi-channel audio is downmixed to mono.
- Samples are stored as `Vec<f32>`.

`AudioClipPlayer` tracks:

- The clip.
- Current sample position.
- Playback step.
- Volume.

`next_sample` returns the next interpolated sample and advances the playback position.

## Sample Rate Differences

The microphone input and VB-Cable output may use different sample rates. The voice output stream handles this with a simple linear interpolation between previous and next mic samples.

The code calculates:

```rust
input_sample_rate / voice_sample_rate
```

as `mic_resample_step`.

## Stats Logger

The engine tracks:

- Dropped input frames.
- Missing output frames.

Every two seconds, the stats thread checks the counters. It records a line in a small rolling log only when there is something meaningful to report, such as dropped input frames or missing output frames. It still writes warnings to stderr when counters are non-zero.

The frontend can poll this rolling log with the `audio_stats_log` Tauri command. When **Show Stats Log** is enabled in the Config tab, the Main tab renders the output in a terminal-style panel.

The rolling log belongs to `Soundboard`, not to a single `AudioEngine` instance, so stopping the audio engine does not clear the visible log. Start/stop events are appended as command-line-style lines.

These logs help diagnose latency or buffer sizing problems.

### Stats Log Messages

The visible stats log is intentionally sparse. It should show lifecycle events and warning conditions, not a constant stream of healthy zero-count samples.

Lifecycle lines:

```text
$ audio engine starting
$ audio stats logger initialized
$ audio engine running
$ audio engine stopping
$ audio engine stopped
```

These confirm the engine and logger lifecycle. If the UI says the engine is running but the log never shows `audio engine running`, inspect `start_audio_engine` and `AudioEngine::start`.

Warning lines:

```text
$ warning dropped_input_frames=N
$ warning missing_output_frames=N
```

`dropped_input_frames` means the input callback could not push mic samples into the ring buffer because the buffer was full. This usually points to timing pressure between input and output, or a buffer/latency mismatch.

Useful things to inspect:

- `LATENCY_MS` in `soundboard.rs`.
- Ring buffer size in `AudioEngine::start`.
- Whether DSP is too expensive for the callback.
- Whether input and output devices have very different timing behavior.

`missing_output_frames` means the output callback needed mic samples but the ring buffer had none available, so it inserted silence. This can produce dropouts or gaps in the mic path.

Useful things to inspect:

- `LATENCY_MS`.
- The selected input device.
- The selected VB-Cable output device.
- Sample-rate mismatch handling.
- Whether the input stream is failing or starving.

The logger still writes more detailed warning text to stderr for backend developers running from a terminal.

## Real-Time Safety

Audio callbacks should avoid:

- Blocking locks.
- File I/O.
- Network I/O.
- Heavy allocation.
- Long logging.

This project follows that mostly by:

- Using channels with `try_recv`.
- Using a ring buffer for mic samples.
- Using atomics for mic effect settings.
- Keeping decoded clips in memory.
