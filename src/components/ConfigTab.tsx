type ConfigTabProps = {
  clipBoostEnabled: boolean;
  monitorClipPlayback: boolean;
  onClipBoostEnabledChange: (enabled: boolean) => void;
  onMonitorClipPlaybackChange: (enabled: boolean) => void;
};

function ConfigTab({
  clipBoostEnabled,
  monitorClipPlayback,
  onClipBoostEnabledChange,
  onMonitorClipPlaybackChange,
}: ConfigTabProps) {
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
    </section>
  );
}

export default ConfigTab;
