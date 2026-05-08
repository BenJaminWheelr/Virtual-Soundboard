type ConfigTabProps = {
  monitorClipPlayback: boolean;
  onMonitorClipPlaybackChange: (enabled: boolean) => void;
};

function ConfigTab({
  monitorClipPlayback,
  onMonitorClipPlaybackChange,
}: ConfigTabProps) {
  return (
    <section className="config-layout">
      <section className="panel config-panel">
        <h2>Playback</h2>
        <label className="switch-field">
          <span>Hear my audio clips</span>
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
      </section>
    </section>
  );
}

export default ConfigTab;
