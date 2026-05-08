import { useState } from "react";
import { gridDimensionOptions } from "../soundboardState";
import type { GridSize, SoundboardCell, UploadedClip } from "../types";

type SoundboardTabProps = {
  busy: boolean;
  cells: SoundboardCell[];
  gridSize: GridSize;
  selectedCellId: string;
  statusEngineRunning: boolean;
  uploadedClips: UploadedClip[];
  onCellChange: (cell: SoundboardCell) => void;
  onGridSizeChange: (size: GridSize) => void;
  onPlayCell: (cell: SoundboardCell) => void;
  onSelectCell: (cellId: string) => void;
};

function SoundboardTab({
  busy,
  cells,
  gridSize,
  selectedCellId,
  statusEngineRunning,
  uploadedClips,
  onCellChange,
  onGridSizeChange,
  onPlayCell,
  onSelectCell,
}: SoundboardTabProps) {
  const [setupOpen, setSetupOpen] = useState(false);
  const selectedCell = cells.find((cell) => cell.id === selectedCellId) ?? cells[0];

  return (
    <section className={setupOpen ? "soundboard-layout drawer-open" : "soundboard-layout"}>
      <button
        aria-label={setupOpen ? "Close soundboard setup" : "Open soundboard setup"}
        aria-expanded={setupOpen}
        className="drawer-toggle"
        type="button"
        onClick={() => setSetupOpen((isOpen) => !isOpen)}
      >
        <span />
        <span />
        <span />
      </button>

      {setupOpen && (
        <button
          aria-label="Close soundboard setup overlay"
          className="drawer-scrim"
          type="button"
          onClick={() => setSetupOpen(false)}
        />
      )}

      <aside className="panel soundboard-drawer">
        <h2>Soundboard Setup</h2>
        <div className="field-pair">
          <label>
            Rows
            <select
              value={gridSize.rows}
              onChange={(event) =>
                onGridSizeChange({ ...gridSize, rows: Number(event.target.value) })
              }
            >
              {gridDimensionOptions.map((dimension) => (
                <option key={dimension} value={dimension}>
                  {dimension}
                </option>
              ))}
            </select>
          </label>

          <label>
            Columns
            <select
              value={gridSize.cols}
              onChange={(event) =>
                onGridSizeChange({ ...gridSize, cols: Number(event.target.value) })
              }
            >
              {gridDimensionOptions.map((dimension) => (
                <option key={dimension} value={dimension}>
                  {dimension}
                </option>
              ))}
            </select>
          </label>
        </div>

        {selectedCell && (
          <div className="cell-editor">
            <h3>Selected Cell</h3>
            <label>
              Text
              <input
                value={selectedCell.label}
                onChange={(event) =>
                  onCellChange({ ...selectedCell, label: event.target.value })
                }
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.currentTarget.blur();
                  }
                }}
              />
            </label>
            <label>
              Audio clip
              <select
                value={selectedCell.clipId}
                onChange={(event) =>
                  onCellChange({ ...selectedCell, clipId: event.target.value })
                }
              >
                <option value="">No clip assigned</option>
                {uploadedClips.map((clip) => (
                  <option key={clip.id} value={clip.id}>
                    {clip.name}
                  </option>
                ))}
              </select>
            </label>
            <button
              className="secondary-button"
              onClick={() => onCellChange({ ...selectedCell, clipId: "" })}
            >
              Remove Clip
            </button>
            <label>
              Clip volume
              <div className="range-field">
                <input
                  min="0.01"
                  max="1"
                  step="0.01"
                  type="range"
                  value={selectedCell.volume}
                  onChange={(event) =>
                    onCellChange({
                      ...selectedCell,
                      volume: Number(event.target.value),
                    })
                  }
                />
                <output>{selectedCell.volume.toFixed(2)}</output>
              </div>
            </label>
            <label>
              Hotkey
              <input
                placeholder="Press a key"
                readOnly
                value={selectedCell.hotkey}
                onKeyDown={(event) => {
                  event.preventDefault();
                  if (event.key === "Enter") {
                    event.currentTarget.blur();
                    return;
                  }

                  const hotkey = formatHotkey(event.nativeEvent);
                  if (hotkey) {
                    onCellChange({ ...selectedCell, hotkey });
                    event.currentTarget.blur();
                  }
                }}
              />
            </label>
            <button
              className="secondary-button"
              disabled={!selectedCell.hotkey}
              onClick={() => onCellChange({ ...selectedCell, hotkey: "" })}
            >
              Remove Keybind
            </button>
          </div>
        )}
      </aside>

      <section
        className="soundboard-grid"
        style={{ gridTemplateColumns: `repeat(${gridSize.cols}, minmax(0, 1fr))` }}
      >
        {cells.map((cell) => {
          const assignedClip = uploadedClips.find((clip) => clip.id === cell.clipId);
          const isSelected = cell.id === selectedCellId;

          return (
            <button
              key={cell.id}
              className={isSelected ? "clip-pad selected" : "clip-pad"}
              aria-disabled={busy || !statusEngineRunning || !assignedClip}
              onClick={() => {
                onSelectCell(cell.id);
                if (!busy && statusEngineRunning && assignedClip) {
                  onPlayCell(cell);
                }
              }}
              onKeyDown={(event) => {
                if (event.key === " ") {
                  event.preventDefault();
                  onSelectCell(cell.id);
                  setSetupOpen((isOpen) => !isOpen);
                  return;
                }

                if (event.key === "Enter") {
                  event.preventDefault();
                }
              }}
              onKeyUp={(event) => {
                if (event.key === " " || event.key === "Enter") {
                  event.preventDefault();
                }
              }}
            >
              <span>{cell.label || "Untitled"}</span>
              <small>
                {assignedClip?.name ?? "Unassigned"}
                {cell.hotkey ? ` - ${cell.hotkey}` : ""}
              </small>
            </button>
          );
        })}
      </section>
    </section>
  );
}

function formatHotkey(event: KeyboardEvent | React.KeyboardEvent) {
  if (["Control", "Shift", "Alt", "Meta", "Tab", "Escape"].includes(event.key)) {
    return "";
  }

  const parts = [];
  if (event.ctrlKey) {
    parts.push("Ctrl");
  }
  if (event.altKey) {
    parts.push("Alt");
  }
  if (event.shiftKey) {
    parts.push("Shift");
  }
  if (event.metaKey) {
    parts.push("Meta");
  }

  const key = event.key === " " ? "Space" : event.key.toUpperCase();
  parts.push(key);
  return parts.join("+");
}

export default SoundboardTab;
