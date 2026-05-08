import type { UploadedClip } from "../types";

type AudioClipManagerTabProps = {
  busy: boolean;
  clips: UploadedClip[];
  uploadMessage: string;
  uploadMessageTone: "error" | "info";
  onDeleteClip: (clipId: string) => void;
  onUploadClips: () => void;
};

function AudioClipManagerTab({
  busy,
  clips,
  uploadMessage,
  uploadMessageTone,
  onDeleteClip,
  onUploadClips,
}: AudioClipManagerTabProps) {
  return (
    <section className="tab-layout">
      <aside className="panel">
        <h2>Upload</h2>
        <button
          className="upload-box"
          disabled={busy}
          type="button"
          onClick={onUploadClips}
        >
          Choose MP3 or WAV files
        </button>
        {uploadMessage && (
          <p className={uploadMessageTone === "error" ? "upload-error" : "upload-info"}>
            {uploadMessage}
          </p>
        )}
      </aside>

      <section className="panel">
        <h2>Uploaded Clips</h2>
        <div className="clip-list">
          {clips.length === 0 && <p className="muted">No clips uploaded yet.</p>}
          {clips.map((clip) => (
            <article key={clip.id} className="clip-row">
              <div className="clip-identity">
                <strong>{clip.name}</strong>
                <span>
                  {clip.file_name} - {clip.format.toUpperCase()}
                </span>
              </div>
              <button
                className="secondary-button"
                disabled={busy}
                onClick={() => onDeleteClip(clip.id)}
              >
                Delete
              </button>
            </article>
          ))}
        </div>
      </section>
    </section>
  );
}

export default AudioClipManagerTab;
