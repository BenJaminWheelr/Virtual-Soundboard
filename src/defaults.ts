import type { GridSize, MicEffectsConfig } from "./types";

export const defaultGridSize: GridSize = {
  rows: 5,
  cols: 5,
};

export const defaultMicEffects: MicEffectsConfig = {
  high_pass: {
    enabled: false,
    cutoff_hz: 80,
  },
  low_pass: {
    enabled: false,
    cutoff_hz: 12000,
  },
  saturation: {
    enabled: false,
    drive: 1.5,
  },
};
