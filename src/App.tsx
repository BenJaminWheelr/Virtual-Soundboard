import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import AudioClipManagerTab from "./components/AudioClipManagerTab";
import ConfigTab from "./components/ConfigTab";
import MainTab from "./components/MainTab";
import SoundboardTab from "./components/SoundboardTab";
import Tabs from "./components/Tabs";
import { createCells } from "./soundboardState";
import type {
  AudioDeviceLists,
  GridSize,
  SoundboardCell,
  SoundboardLayout,
  SoundboardStatus,
  TabId,
  UploadedClip,
} from "./types";

const fallbackStatus: SoundboardStatus = {
  engine_running: false,
  clips_dir: "",
  clip_count: 0,
};

const fallbackDevices: AudioDeviceLists = {
  inputs: [],
  outputs: [],
  vb_cable: {
    installed: false,
    playback_device: null,
    voice_chat_input_name: "CABLE Output (VB-Audio Virtual Cable)",
  },
  monitor_output: null,
};

const defaultGridSize: GridSize = {
  rows: 5,
  cols: 5,
};

function App() {
  const [activeTab, setActiveTab] = useState<TabId>("main");
  const [busy, setBusy] = useState(false);
  const [devices, setDevices] = useState<AudioDeviceLists>(fallbackDevices);
  const [gridSize, setGridSize] = useState<GridSize>(defaultGridSize);
  const [layoutLoaded, setLayoutLoaded] = useState(false);
  const [initialDataLoaded, setInitialDataLoaded] = useState(false);
  const [message, setMessage] = useState("Backend idle");
  const [clipBoostEnabled, setClipBoostEnabled] = useState(false);
  const [micTestLevel, setMicTestLevel] = useState(0);
  const [micTestRunning, setMicTestRunning] = useState(false);
  const [monitorClipPlayback, setMonitorClipPlayback] = useState(true);
  const [selectedCellId, setSelectedCellId] = useState("cell-0");
  const [selectedInput, setSelectedInput] = useState("");
  const [selectedMonitorOutput, setSelectedMonitorOutput] = useState("");
  const [status, setStatus] = useState<SoundboardStatus>(fallbackStatus);
  const [uploadedClips, setUploadedClips] = useState<UploadedClip[]>([]);
  const [uploadMessage, setUploadMessage] = useState("");
  const [uploadMessageTone, setUploadMessageTone] = useState<"error" | "info">("info");
  const [cells, setCells] = useState<SoundboardCell[]>(() =>
    createCells(defaultGridSize),
  );

  const selectedCell = useMemo(
    () => cells.find((cell) => cell.id === selectedCellId),
    [cells, selectedCellId],
  );

  useEffect(() => {
    loadInitialData();
  }, []);

  useEffect(() => {
    updateGlobalHotkeys();
  }, [cells]);

  useEffect(() => {
    if (!micTestRunning) {
      setMicTestLevel(0);
      return;
    }

    const intervalId = window.setInterval(async () => {
      try {
        const nextLevel = await invoke<number>("mic_test_level");
        setMicTestLevel(nextLevel);
      } catch (error) {
        setMessage(formatError(error));
      }
    }, 60);

    return () => window.clearInterval(intervalId);
  }, [micTestRunning]);

  useEffect(() => {
    if (!layoutLoaded) {
      return;
    }

    const saveTimeout = window.setTimeout(() => {
      saveSoundboardLayout();
    }, 250);

    return () => window.clearTimeout(saveTimeout);
  }, [
    cells,
    clipBoostEnabled,
    gridSize,
    monitorClipPlayback,
    selectedInput,
    selectedMonitorOutput,
    layoutLoaded,
  ]);

  async function refreshStatus() {
    try {
      const nextStatus = await invoke<SoundboardStatus>("backend_status");
      setStatus(nextStatus);
    } catch (error) {
      setMessage(formatError(error));
    }
  }

  async function loadInitialData() {
    await Promise.all([
      refreshStatus(),
      loadDevices(),
      loadClips(),
      loadSoundboardLayout(),
    ]);
    setInitialDataLoaded(true);
  }

  async function loadDevices() {
    try {
      const nextDevices = await invoke<AudioDeviceLists>("list_audio_devices");
      setDevices(nextDevices);
    } catch (error) {
      setMessage(formatError(error));
    }
  }

  async function refreshDevices() {
    setBusy(true);
    setMessage("Scanning audio devices...");

    try {
      await loadDevices();
      setMessage("Audio devices refreshed");
    } finally {
      setBusy(false);
    }
  }

  async function loadSoundboardLayout() {
    try {
      const savedLayout = await invoke<SoundboardLayout | null>(
        "load_soundboard_layout",
      );

      if (savedLayout && isValidGridSize(savedLayout.grid_size)) {
        setGridSize(savedLayout.grid_size);
        setCells(
          createCells(
            savedLayout.grid_size,
            Array.isArray(savedLayout.cells) ? savedLayout.cells : [],
          ),
        );
        changeClipBoostEnabled(savedLayout.clip_boost_enabled ?? false, false);
        changeMonitorClipPlayback(savedLayout.monitor_clip_playback ?? true, false);
        setSelectedInput(savedLayout.selected_input ?? "");
        setSelectedMonitorOutput(savedLayout.selected_monitor_output ?? "");
      }
    } catch (error) {
      setMessage(formatError(error));
    } finally {
      setLayoutLoaded(true);
    }
  }

  async function saveSoundboardLayout(overrides: Partial<SoundboardLayout> = {}) {
    try {
      await invoke("save_soundboard_layout", {
        layout: {
          grid_size: gridSize,
          cells,
          clip_boost_enabled: clipBoostEnabled,
          monitor_clip_playback: monitorClipPlayback,
          selected_input: selectedInput,
          selected_monitor_output: selectedMonitorOutput,
          ...overrides,
        },
      });
    } catch (error) {
      setMessage(formatError(error));
    }
  }

  function changeSelectedInput(device: string) {
    setSelectedInput(device);
    if (layoutLoaded) {
      saveSoundboardLayout({ selected_input: device });
    }
  }

  function changeSelectedMonitorOutput(device: string) {
    setSelectedMonitorOutput(device);
    if (layoutLoaded) {
      saveSoundboardLayout({ selected_monitor_output: device });
    }
  }

  async function changeMonitorClipPlayback(enabled: boolean, saveImmediately = true) {
    setMonitorClipPlayback(enabled);

    try {
      await invoke("set_monitor_clip_playback", { enabled });
      if (saveImmediately && layoutLoaded) {
        saveSoundboardLayout({ monitor_clip_playback: enabled });
      }
    } catch (error) {
      setMessage(formatError(error));
    }
  }

  async function changeClipBoostEnabled(enabled: boolean, saveImmediately = true) {
    setClipBoostEnabled(enabled);

    try {
      await invoke("set_clip_boost_enabled", { enabled });
      if (saveImmediately && layoutLoaded) {
        saveSoundboardLayout({ clip_boost_enabled: enabled });
      }
    } catch (error) {
      setMessage(formatError(error));
    }
  }

  async function startAudioEngine() {
    setBusy(true);
    setMessage("Starting audio engine...");

    try {
      if (micTestRunning) {
        await invoke("stop_mic_test");
        setMicTestRunning(false);
      }

      const nextStatus = await invoke<SoundboardStatus>("start_audio_engine", {
        selection: {
          input_device: selectedInput || null,
          monitor_output_device: selectedMonitorOutput || null,
        },
      });
      setStatus(nextStatus);
      setMessage("Audio engine running");
    } catch (error) {
      setMessage(formatError(error));
    } finally {
      setBusy(false);
    }
  }

  async function toggleMicTest() {
    setBusy(true);

    try {
      if (micTestRunning) {
        await invoke("stop_mic_test");
        setMicTestRunning(false);
        setMessage("Microphone test stopped");
        return;
      }

      await invoke("start_mic_test", {
        inputDevice: selectedInput || null,
      });
      setMicTestRunning(true);
      setMessage("Microphone test running");
    } catch (error) {
      setMessage(formatError(error));
    } finally {
      setBusy(false);
    }
  }
  
  async function stopAudioEngine() {
    setBusy(true);
    setMessage("Stopping audio engine...");

    try {
      // This assumes you have a 'stop_audio_engine' command in your Rust backend
      const nextStatus = await invoke<SoundboardStatus>("stop_audio_engine");
      setStatus(nextStatus);
      setMessage("Audio engine stopped");
    } catch (error) {
      setMessage(formatError(error));
    } finally {
      setBusy(false);
    }
  }

  async function playCell(cell: SoundboardCell) {
    const clip = uploadedClips.find((uploadedClip) => uploadedClip.id === cell.clipId);
    if (!clip) {
      setMessage("Assign a clip to this cell first");
      return;
    }

    setSelectedCellId(cell.id);
    setBusy(true);
    setMessage(`Triggering ${clip.name}...`);

    try {
      const nextStatus = await invoke<SoundboardStatus>("play_clip", {
        clipId: clip.id,
        volume: cell.volume,
      });
      setStatus(nextStatus);
      setMessage(`${cell.label || clip.name} triggered`);
    } catch (error) {
      setMessage(formatError(error));
    } finally {
      setBusy(false);
    }
  }

  function changeGridSize(nextSize: GridSize) {
    setGridSize(nextSize);
    setCells((currentCells) => createCells(nextSize, currentCells));
    setSelectedCellId("cell-0");
  }

  function updateCell(nextCell: SoundboardCell) {
    setCells((currentCells) =>
      currentCells.map((cell) => (cell.id === nextCell.id ? nextCell : cell)),
    );
  }

  async function loadClips() {
    try {
      const clips = await invoke<UploadedClip[]>("list_clips");
      setUploadedClips(clips);
    } catch (error) {
      setMessage(formatError(error));
    }
  }

  async function updateGlobalHotkeys() {
    const bindings = cells
      .filter((cell) => cell.hotkey && cell.clipId)
      .map((cell) => ({
        hotkey: cell.hotkey,
        clip_id: cell.clipId,
        volume: cell.volume,
      }));

    try {
      await invoke("update_global_hotkeys", { bindings });
    } catch (error) {
      setMessage(formatError(error));
    }
  }

  async function uploadClips() {
    setUploadMessage("");
    setUploadMessageTone("info");

    let supportedPaths: string[];

    try {
      const selectedPaths = await open({
        multiple: true,
        filters: [
          {
            name: "Audio clips",
            extensions: ["mp3", "wav"],
          },
        ],
      });

      if (!selectedPaths) {
        return;
      }

      const paths = Array.isArray(selectedPaths) ? selectedPaths : [selectedPaths];
      supportedPaths = paths.filter((path) => isSupportedAudioFile(path));
    } catch (error) {
      setUploadMessage(formatError(error));
      setUploadMessageTone("error");
      return;
    }

    if (supportedPaths.length === 0) {
      setUploadMessage("Choose MP3 or WAV files.");
      setUploadMessageTone("error");
      return;
    }

    setBusy(true);
    setMessage("Importing clips...");
    setUploadMessage("Importing clips...");
    setUploadMessageTone("info");

    try {
      for (const sourcePath of supportedPaths) {
        await invoke<UploadedClip>("import_clip", {
          sourcePath,
        });
      }

      await loadClips();
      await refreshStatus();
      setUploadMessage(
        `${supportedPaths.length} clip${supportedPaths.length === 1 ? "" : "s"} imported.`,
      );
      setUploadMessageTone("info");
      setMessage(
        `${supportedPaths.length} clip${supportedPaths.length === 1 ? "" : "s"} imported`,
      );
    } catch (error) {
      const errorMessage = formatError(error);
      setUploadMessage(errorMessage);
      setUploadMessageTone("error");
      setMessage("Clip import failed");
    } finally {
      setBusy(false);
    }
  }

  async function deleteClip(clipId: string) {
    setBusy(true);
    setMessage("Deleting clip...");

    try {
      await invoke("delete_clip", { clipId });
      setUploadedClips((currentClips) =>
        currentClips.filter((clip) => clip.id !== clipId),
      );
      setCells((currentCells) =>
        currentCells.map((cell) =>
          cell.clipId === clipId ? { ...cell, clipId: "" } : cell,
        ),
      );
      await refreshStatus();
      setMessage("Clip deleted");
    } catch (error) {
      setMessage(formatError(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="app-shell">
      <section className="top-bar">
        <div>
          <h1>Virtual Soundboard</h1>
          <p>{message}</p>
        </div>
        <span className={status.engine_running ? "status live" : "status"}>
          {status.engine_running ? "Engine Live" : "Engine Off"}
        </span>
      </section>

      {!initialDataLoaded ? (
        <section className="loading-panel" aria-live="polite">
          <div className="loading-mark" />
          <h2>Loading soundboard</h2>
          <p>Restoring clips, devices, and saved layout.</p>
        </section>
      ) : (
        <>
          <Tabs activeTab={activeTab} onChange={setActiveTab} />

          {activeTab === "main" && (
            <MainTab
              busy={busy}
              devices={devices}
              micTestLevel={micTestLevel}
              micTestRunning={micTestRunning}
              selectedInput={selectedInput}
              selectedMonitorOutput={selectedMonitorOutput}
              status={status}
          onInputChange={changeSelectedInput}
          onMonitorOutputChange={changeSelectedMonitorOutput}
              onRefreshDevices={refreshDevices}
              onStartAudioEngine={status.engine_running ? stopAudioEngine : startAudioEngine}
              onToggleMicTest={toggleMicTest}
            />
          )}

          {activeTab === "soundboard" && (
            <SoundboardTab
              busy={busy}
              cells={cells}
              gridSize={gridSize}
              selectedCellId={selectedCell?.id ?? "cell-0"}
              statusEngineRunning={status.engine_running}
              uploadedClips={uploadedClips}
              onCellChange={updateCell}
              onGridSizeChange={changeGridSize}
              onPlayCell={playCell}
              onSelectCell={setSelectedCellId}
            />
          )}

          {activeTab === "clips" && (
            <AudioClipManagerTab
              busy={busy}
              clips={uploadedClips}
              uploadMessage={uploadMessage}
              uploadMessageTone={uploadMessageTone}
              onDeleteClip={deleteClip}
              onUploadClips={uploadClips}
            />
          )}

          {activeTab === "config" && (
            <ConfigTab
              clipBoostEnabled={clipBoostEnabled}
              monitorClipPlayback={monitorClipPlayback}
              onClipBoostEnabledChange={changeClipBoostEnabled}
              onMonitorClipPlaybackChange={changeMonitorClipPlayback}
            />
          )}
        </>
      )}
    </main>
  );
}

function isSupportedAudioFile(fileName: string) {
  return /\.(mp3|wav)$/i.test(fileName);
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function isValidGridSize(size: unknown): size is GridSize {
  if (!size || typeof size !== "object") {
    return false;
  }

  const candidate = size as GridSize;
  return (
    Number.isInteger(candidate.rows) &&
    Number.isInteger(candidate.cols) &&
    candidate.rows >= 1 &&
    candidate.rows <= 5 &&
    candidate.cols >= 1 &&
    candidate.cols <= 5
  );
}

export default App;
