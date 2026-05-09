type ConfigTabProps = {
  earRapeEnabled: boolean;
  monitorClipPlayback: boolean;
  onEarRapeEnabledChange: (enabled: boolean) => void;
  onMonitorClipPlaybackChange: (enabled: boolean) => void;
};

function ConfigTab({
  earRapeEnabled,
  monitorClipPlayback,
  onEarRapeEnabledChange,
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
          <span className="setting-title">Ear rape</span>
          <input
            checked={earRapeEnabled}
            type="checkbox"
            onChange={(event) => onEarRapeEnabledChange(event.target.checked)}
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
