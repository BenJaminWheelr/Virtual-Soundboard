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
  showStatsLog: boolean;
  onClipBoostEnabledChange: (enabled: boolean) => void;
  onMicEffectsChange: (config: MicEffectsConfig) => void;
  onMonitorClipPlaybackChange: (enabled: boolean) => void;
  onShowStatsLogChange: (enabled: boolean) => void;
};

function ConfigTab({
  clipBoostEnabled,
  micEffects,
  monitorClipPlayback,
  showStatsLog,
  onClipBoostEnabledChange,
  onMicEffectsChange,
  onMonitorClipPlaybackChange,
  onShowStatsLogChange,
}: ConfigTabProps) {
  function updateMicEffects(config: Partial<MicEffectsConfig>) {
    onMicEffectsChange({
      ...micEffects,
      ...config,
    });
  }

  function updateEffect<Key extends keyof MicEffectsConfig>(
    key: Key,
    config: Partial<MicEffectsConfig[Key]>,
  ) {
    updateMicEffects({
      [key]: {
        ...micEffects[key],
        ...config,
      },
    });
  }

  return (
    <section className="config-layout">
      <section className="panel config-panel">
        <h2>Playback</h2>
        <SettingSection
          enabled={monitorClipPlayback}
          title="Hear my audio clips"
          onEnabledChange={onMonitorClipPlaybackChange}
        >
          When off, clips still go to voice chat but do not play through your
          selected monitor output.
        </SettingSection>

        <SettingSection
          danger
          enabled={clipBoostEnabled}
          title="Clip Boost"
          onEnabledChange={onClipBoostEnabledChange}
        >
          Boosts every audio clip's volume.
        </SettingSection>

        <SettingSection
          enabled={showStatsLog}
          title="Show Stats Log"
          onEnabledChange={onShowStatsLogChange}
        >
          Shows the audio engine stats output on the Main tab.
        </SettingSection>
      </section>

      <section className="panel config-panel">
        <h2>Effects</h2>

        <EffectSection
          enabled={micEffects.high_pass.enabled}
          title="High-Pass Filter"
          onEnabledChange={(enabled) => updateEffect("high_pass", { enabled })}
        >
          <EffectRange
            label="Cutoff"
            min={20}
            max={2000}
            step={10}
            value={micEffects.high_pass.cutoff_hz}
            wideOutput
            formatValue={(value) => `${Math.round(value)} Hz`}
            onChange={(cutoff_hz) => updateEffect("high_pass", { cutoff_hz })}
          />
        </EffectSection>

        <EffectSection
          enabled={micEffects.low_pass.enabled}
          title="Low-Pass Filter"
          onEnabledChange={(enabled) => updateEffect("low_pass", { enabled })}
        >
          <EffectRange
            label="Cutoff"
            min={1000}
            max={20000}
            step={100}
            value={micEffects.low_pass.cutoff_hz}
            wideOutput
            formatValue={(value) => `${Math.round(value)} Hz`}
            onChange={(cutoff_hz) => updateEffect("low_pass", { cutoff_hz })}
          />
        </EffectSection>

        <EffectSection
          enabled={micEffects.saturation.enabled}
          title="Soft Saturation"
          onEnabledChange={(enabled) => updateEffect("saturation", { enabled })}
        >
          <EffectRange
            label="Drive"
            min={1}
            max={8}
            step={0.1}
            value={micEffects.saturation.drive}
            formatValue={(value) => value.toFixed(1)}
            onChange={(drive) => updateEffect("saturation", { drive })}
          />
        </EffectSection>
      </section>
    </section>
  );
}

function SettingSection({
  children,
  danger = false,
  enabled,
  title,
  onEnabledChange,
}: {
  children: ReactNode;
  danger?: boolean;
  enabled: boolean;
  title: string;
  onEnabledChange: (enabled: boolean) => void;
}) {
  return (
    <section className="setting-section">
      <label className={danger ? "switch-field danger-switch" : "switch-field"}>
        <span className="setting-title">{title}</span>
        <input
          checked={enabled}
          type="checkbox"
          onChange={(event) => onEnabledChange(event.target.checked)}
        />
        <span className="switch-slider" aria-hidden="true" />
      </label>
      <p className="muted">{children}</p>
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
