import type { ReactNode } from "react";
import type { MicEffectsConfig } from "../types";

type EffectRangeProps = {
  label: string;
  max: number;
  min: number;
  step: number;
  value: number;
  wideOutput?: boolean;
  formatValue: (value: number) => string;
  onChange: (value: number) => void;
};

type EffectSectionProps = {
  children: ReactNode;
  enabled: boolean;
  title: string;
  onEnabledChange: (enabled: boolean) => void;
};

type ConfigTabProps = {
  clipBoostEnabled: boolean;
  micEffects: MicEffectsConfig;
  monitorClipPlayback: boolean;
  onClipBoostEnabledChange: (enabled: boolean) => void;
  onMicEffectsChange: (config: MicEffectsConfig) => void;
  onMonitorClipPlaybackChange: (enabled: boolean) => void;
};

function ConfigTab({
  clipBoostEnabled,
  micEffects,
  monitorClipPlayback,
  onClipBoostEnabledChange,
  onMicEffectsChange,
  onMonitorClipPlaybackChange,
}: ConfigTabProps) {
  function updateMicEffects(config: Partial<MicEffectsConfig>) {
    onMicEffectsChange({
      ...micEffects,
      ...config,
    });
  }

  return (
    <section className="config-layout">
      <section className="panel config-panel">
        <h2>Playback</h2>
        <label className="switch-field">
          <span className="setting-title">Hear my audio clips</span>
          <input
            checked={monitorClipPlayback}
            type="checkbox"
            onChange={(event) => onMonitorClipPlaybackChange(event.target.checked)}
          />
          <span className="switch-slider" aria-hidden="true" />
        </label>
        <p className="muted">
          When off, clips still go to voice chat but do not play through your
          selected monitor output.
        </p>

        <label className="switch-field danger-switch">
          <span className="setting-title">Clip Boost</span>
          <input
            checked={clipBoostEnabled}
            type="checkbox"
            onChange={(event) => onClipBoostEnabledChange(event.target.checked)}
          />
          <span className="switch-slider" aria-hidden="true" />
        </label>
        <p className="muted">
          Boosts every audio clip's volume.
        </p>
      </section>

      <section className="panel config-panel">
        <h2>Effects</h2>

        <EffectSection
          enabled={micEffects.noise_gate.enabled}
          title="Noise Gate"
          onEnabledChange={(enabled) =>
            updateMicEffects({
              noise_gate: { ...micEffects.noise_gate, enabled },
            })
          }
        >
          <EffectRange
            label="Threshold"
            min={0}
            max={0.25}
            step={0.005}
            value={micEffects.noise_gate.threshold}
            formatValue={(value) => value.toFixed(3)}
            onChange={(threshold) =>
              updateMicEffects({
                noise_gate: { ...micEffects.noise_gate, threshold },
              })
            }
          />
        </EffectSection>

        <EffectSection
          enabled={micEffects.high_pass.enabled}
          title="High-Pass Filter"
          onEnabledChange={(enabled) =>
            updateMicEffects({
              high_pass: { ...micEffects.high_pass, enabled },
            })
          }
        >
          <EffectRange
            label="Cutoff"
            min={20}
            max={2000}
            step={10}
            value={micEffects.high_pass.cutoff_hz}
            wideOutput
            formatValue={(value) => `${Math.round(value)} Hz`}
            onChange={(cutoff_hz) =>
              updateMicEffects({
                high_pass: { ...micEffects.high_pass, cutoff_hz },
              })
            }
          />
        </EffectSection>

        <EffectSection
          enabled={micEffects.low_pass.enabled}
          title="Low-Pass Filter"
          onEnabledChange={(enabled) =>
            updateMicEffects({
              low_pass: { ...micEffects.low_pass, enabled },
            })
          }
        >
          <EffectRange
            label="Cutoff"
            min={1000}
            max={20000}
            step={100}
            value={micEffects.low_pass.cutoff_hz}
            wideOutput
            formatValue={(value) => `${Math.round(value)} Hz`}
            onChange={(cutoff_hz) =>
              updateMicEffects({
                low_pass: { ...micEffects.low_pass, cutoff_hz },
              })
            }
          />
        </EffectSection>

        <EffectSection
          enabled={micEffects.saturation.enabled}
          title="Soft Saturation"
          onEnabledChange={(enabled) =>
            updateMicEffects({
              saturation: { ...micEffects.saturation, enabled },
            })
          }
        >
          <EffectRange
            label="Drive"
            min={1}
            max={8}
            step={0.1}
            value={micEffects.saturation.drive}
            formatValue={(value) => value.toFixed(1)}
            onChange={(drive) =>
              updateMicEffects({
                saturation: { ...micEffects.saturation, drive },
              })
            }
          />
        </EffectSection>
      </section>
    </section>
  );
}

function EffectSection({
  children,
  enabled,
  title,
  onEnabledChange,
}: EffectSectionProps) {
  return (
    <section className="effect-section">
      <label className="switch-field">
        <span className="setting-title">{title}</span>
        <input
          checked={enabled}
          type="checkbox"
          onChange={(event) => onEnabledChange(event.target.checked)}
        />
        <span className="switch-slider" aria-hidden="true" />
      </label>
      {children}
    </section>
  );
}

function EffectRange({
  label,
  max,
  min,
  step,
  value,
  wideOutput = false,
  formatValue,
  onChange,
}: EffectRangeProps) {
  return (
    <label>
      {label}
      <div className={wideOutput ? "range-field wide-output" : "range-field"}>
        <input
          min={min}
          max={max}
          step={step}
          type="range"
          value={value}
          onChange={(event) => onChange(Number(event.target.value))}
        />
        <output>{formatValue(value)}</output>
      </div>
    </label>
  );
}

export default ConfigTab;
