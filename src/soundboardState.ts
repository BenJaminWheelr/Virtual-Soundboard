import type { GridSize, SoundboardCell } from "./types";

export const gridDimensionOptions = [1, 2, 3, 4, 5];

export function createCells(size: GridSize, existingCells: SoundboardCell[] = []) {
  const existingById = new Map(
    existingCells.map((cell) => [
      cell.id,
      {
        ...cell,
        volume: clampCellVolume(cell.volume),
      },
    ]),
  );
  const cells: SoundboardCell[] = [];

  for (let index = 0; index < size.rows * size.cols; index += 1) {
    const id = `cell-${index}`;
    cells.push(
      existingById.get(id) ?? {
        id,
        label: `Pad ${index + 1}`,
        clipId: "",
        hotkey: "",
        volume: 1,
      },
    );
  }

  return cells;
}

export function clampCellVolume(volume: number | undefined) {
  if (typeof volume !== "number" || Number.isNaN(volume)) {
    return 1;
  }

  return Math.min(1, Math.max(0.01, volume));
}
