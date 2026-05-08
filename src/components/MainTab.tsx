import type { AudioDeviceInfo, AudioDeviceLists, SoundboardStatus } from "../types";

type MainTabProps = {
  busy: boolean;
  devices: AudioDeviceLists;
  micTestLevel: number;
  micTestRunning: boolean;
  selectedInput: string;
  selectedMonitorOutput: string;
  status: SoundboardStatus;
  onInputChange: (device: string) => void;
  onMonitorOutputChange: (device: string) => void;
  onRefreshDevices: () => void;
  onStartAudioEngine: () => void;
  onToggleMicTest: () => void;
};

function MainTab({
  busy,
  devices,
  micTestLevel,
  micTestRunning,
  selectedInput,
  selectedMonitorOutput,
  status,
  onInputChange,
  onMonitorOutputChange,
  onRefreshDevices,
  onStartAudioEngine,
  onToggleMicTest,
}: MainTabProps) {
  const selectedInputInfo = selectedInput
    ? devices.inputs.find((device) => device.name === selectedInput)
    : devices.inputs[0];
  const selectedMonitorOutputInfo = selectedMonitorOutput
    ? devices.outputs.find((device) => device.name === selectedMonitorOutput)
    : devices.monitor_output ?? devices.outputs[0];

  return (
    <section className="tab-layout">
      <aside className="panel">
        <h2>Engine</h2>
        <button
          className="primary-button"
          disabled={busy}
          onClick={onStartAudioEngine}
        >
          {status.engine_running ? "Stop Audio Engine" : "Start Audio Engine"}
        </button>
        <dl>
          <div>
            <dt>State</dt>
            <dd>{status.engine_running ? "Running" : "Stopped"}</dd>
          </div>
          <div>
            <dt>Imported clips</dt>
            <dd>{status.clip_count}</dd>
          </div>
          <div>
            <dt>Clip folder</dt>
            <dd>{status.clips_dir || "Loading..."}</dd>
          </div>
        </dl>

        <div className="mic-test">
          <div className="panel-heading">
            <h2>Mic Test</h2>
            <button
              className="secondary-button compact-button"
              disabled={busy || status.engine_running}
              type="button"
              onClick={onToggleMicTest}
            >
              {micTestRunning ? "Stop" : "Test"}
            </button>
          </div>
          <div className="level-track" aria-label="Microphone input level">
            <div
              className="level-fill"
              style={{ width: `${Math.round(micTestLevel * 100)}%` }}
            />
          </div>
          <p className="muted">
            {micTestRunning ? "Listening to selected input." : "Test selected input level."}
          </p>
        </div>
      </aside>

      <section className="panel">
        <div className="panel-heading">
          <h2>Routing</h2>
          <button
            className="secondary-button compact-button"
            disabled={busy || status.engine_running}
            type="button"
            onClick={onRefreshDevices}
          >
            Scan
          </button>
        </div>
        <div className="field-grid">
          <label>
            Input
            <select
              disabled={status.engine_running}
              value={selectedInput}
              onChange={(event) => onInputChange(event.target.value)}
            >
              <option value="">System default input</option>
              {devices.inputs.map((device) => (
                <option key={device.name} value={device.name}>
                  {device.name}
                </option>
              ))}
            </select>
          </label>

          <label>
            Clip monitor output
            <select
              disabled={status.engine_running}
              value={selectedMonitorOutput}
              onChange={(event) => onMonitorOutputChange(event.target.value)}
            >
              <option value="">System default speakers/headphones</option>
              {devices.outputs.map((device) => (
                <option key={device.name} value={device.name}>
                  {device.name}
                </option>
              ))}
            </select>
          </label>
        </div>

        <VbCableNotice devices={devices} />

        <div className="device-details">
          <DeviceDetails title="Input Details" device={selectedInputInfo} />
          <DeviceDetails
            title="Clip Monitor"
            device={selectedMonitorOutputInfo ?? undefined}
          />
        </div>
      </section>
    </section>
  );
}

function VbCableNotice({ devices }: { devices: AudioDeviceLists }) {
  if (devices.vb_cable.installed) {
    return (
      <div className="route-notice">
        <strong>VB-Cable detected.</strong>
        <span>
          Voice chat mix goes to {devices.vb_cable.playback_device?.name}. Set
          Discord or your game input to {devices.vb_cable.voice_chat_input_name}.
        </span>
        <span>
          Turn off echo cancellation and noise suppression, and lower the noise
          gate to -90 dB so voice chat does not cut out your audio clips.
        </span>
      </div>
    );
  }

  return (
    <div className="route-notice warning">
      <strong>VB-Cable is not installed.</strong>
      <span>
        Download VB-Cable from VB-Audio, install it as administrator, restart
        this app, then set voice chat input to CABLE Output.
      </span>
    </div>
  );
}

function DeviceDetails({
  device,
  title,
}: {
  device: AudioDeviceInfo | undefined;
  title: string;
}) {
  if (!device) {
    return (
      <article className="device-card">
        <h3>{title}</h3>
        <p className="muted">No device information available.</p>
      </article>
    );
  }

  return (
    <article className="device-card">
      <h3>{title}</h3>
      <dl>
        <div>
          <dt>Name</dt>
          <dd>{device.name}</dd>
        </div>
        <div>
          <dt>Sample rate</dt>
          <dd>{device.sample_rate} Hz</dd>
        </div>
        <div>
          <dt>Channels</dt>
          <dd>{device.channels}</dd>
        </div>
        <div>
          <dt>Sample format</dt>
          <dd>{device.sample_format}</dd>
        </div>
      </dl>
    </article>
  );
}

export default MainTab;
