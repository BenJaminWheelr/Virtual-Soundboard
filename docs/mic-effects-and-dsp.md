# Mic Effects And DSP

Mic effects are applied only to live microphone audio. They do not affect soundboard clips.

The effect chain currently lives in:

- `src-tauri/src/effects/mod.rs`
- `src-tauri/src/effects/simple.rs`

The input stream calls the effect chain in `src-tauri/src/audio.rs`.

## Where Effects Happen

The mic path is:

```text
physical mic
-> CPAL input callback
-> mono downmix
-> MicEffectsProcessor
-> ring buffer
-> voice output stream
-> VB-Cable
```

Clips are mixed later in the output stream, so effects do not process clips.

## Config Flow

The frontend stores effect settings in `MicEffectsConfig`.

When a user changes an effect control:

```text
ConfigTab
-> App.changeMicEffectsConfig
-> invoke("set_mic_effects_config")
-> lib.rs command
-> Soundboard.set_mic_effects_config
-> SharedMicEffects.set_config
```

`SharedMicEffects` stores values in atomics. The audio callback reads snapshots from those atomics.

This avoids locking a normal mutex in the audio callback.

## Effect Chain

`MicEffectsProcessor` orchestrates the chain:

```text
High-Pass Filter
-> Low-Pass Filter
-> Saturation
```

Each effect only runs if enabled in the current snapshot.

## SharedMicEffects

`SharedMicEffects` is the real-time-safe shared config object.

It stores:

- Booleans as `AtomicBool`.
- Floating point values as `AtomicU32` containing `f32::to_bits()`.
- Enum values as `AtomicU32`.

Why this shape?

Rust does not have a standard `AtomicF32`. Storing float bits in `AtomicU32` lets the UI thread update values while the audio thread reads them without a blocking lock.

Auto-Tune DSP is not currently implemented or exposed in the UI. Old saved layout files may still contain an `auto_tune` field, but the current config model ignores it.

## MicEffectsSnapshot

The audio callback reads a snapshot:

```rust
let snapshot = self.effects.snapshot();
```

The snapshot is a plain copy of the current live settings. The effect processor uses the snapshot for one sample.

Only active DSP settings are included in the snapshot. Removed or placeholder config should not be read by the real-time path.

## Simple Effects

`effects/simple.rs` contains the current DSP blocks.

### High-Pass Filter

The high-pass filter reduces low-frequency rumble.

It uses a simple first-order filter. It is not a surgical EQ, but it is cheap and stable for real-time voice cleanup.

### Low-Pass Filter

The low-pass filter reduces high-frequency harshness.

It also uses a first-order filter.

### Soft Saturation

Saturation applies a `tanh` curve:

```rust
(sample * drive).tanh() / drive.tanh()
```

This adds gentle nonlinear color and soft limiting.

## Auto-Tune Status

The previous in-house Auto-Tune implementation was removed after quality testing.

The main failure mode was repeated phonemes and stuttered speech when the processor tried to pitch-correct normal speech. That means the implementation was not acceptable as a foundation for this app.

Auto-Tune should not be re-added as a quick sample-by-sample trick. A good implementation needs at least:

- Reliable voiced/unvoiced detection.
- Stable pitch tracking across frames.
- Key and scale quantization.
- A high-quality pitch shifter or vocal resynthesis algorithm.
- Bypass behavior for consonants, breath, noise, and uncertain pitch.
- Careful latency and buffer ownership so the audio callback remains stable.

The next version should use a proven DSP library for the pitch shifting/resynthesis part instead of custom ad hoc code.

## Candidate Auto-Tune Architecture

There does not appear to be a mature, single Rust crate that implements full musical Auto-Tune end to end.

A better design is a small pipeline:

```text
input frame
-> voiced/unvoiced gate
-> pitch detector
-> key/scale quantizer
-> library pitch shifter
-> smoothed output frame
```

The app should own the musical logic: root note, scale, correction strength, correction speed, and bypass rules.

A crate should own the hard DSP part: pitch shifting or time-stretch/resynthesis.

Promising library roles:

- `pitch_detection`: pitch detection only.
- `ssstretch`: Rust bindings for Signalsmith Stretch, useful for high-quality pitch shifting.
- `pitch_shift`: pure Rust phase-vocoder pitch shifting, easier to build but likely lower confidence for polished vocal correction.
- Rubber Band Library: high-quality C++ option, but licensing and native integration need careful review before shipping.

## Adding A New Effect

A good path for adding a new mic effect:

1. Add config fields to `MicEffectsConfig` in `effects/mod.rs`.
2. Add matching TypeScript fields in `src/types.ts`.
3. Add defaults in `src/defaults.ts`.
4. Add controls in `ConfigTab.tsx`.
5. Add atomic storage in `SharedMicEffects`.
6. Add snapshot fields in `MicEffectsSnapshot`.
7. Add DSP code in either `simple.rs` or a new file.
8. Call the effect from `MicEffectsProcessor.process_sample`.
9. Update docs.

If the effect needs history, store that history in the processor, not in shared config.

## Real-Time Callback Rule

Do not add this kind of work inside the audio callback:

- File reads/writes.
- Network calls.
- Blocking mutex locks.
- Long allocations on every sample.
- Heavy logging.

If an effect needs buffers, allocate them in the processor constructor and reuse them.
