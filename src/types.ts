export type TabId = "main" | "soundboard" | "clips" | "config";

export type SoundboardStatus = {
  engine_running: boolean;
  clips_dir: string;
  clip_count: number;
};

export type AudioDeviceLists = {
  inputs: AudioDeviceInfo[];
  outputs: AudioDeviceInfo[];
  vb_cable: VbCableStatus;
  monitor_output: AudioDeviceInfo | null;
};

export type AudioDeviceInfo = {
  name: string;
  channels: number;
  sample_rate: number;
  sample_format: string;
};

export type VbCableStatus = {
  installed: boolean;
  playback_device: AudioDeviceInfo | null;
  voice_chat_input_name: string;
};

export type UploadedClip = {
  id: string;
  name: string;
  file_name: string;
  format: "mp3" | "wav";
  path: string;
};

export type GridSize = {
  rows: number;
  cols: number;
};

export type SoundboardCell = {
  id: string;
  label: string;
  clipId: string;
  hotkey: string;
  volume: number;
};

export type SoundboardLayout = {
  grid_size: GridSize;
  cells: SoundboardCell[];
  monitor_clip_playback: boolean;
  selected_input: string;
  selected_monitor_output: string;
};
