import { useEffect, useState } from "react";
import { editorParseBody, editorUpdateMeta } from "../lib/tauri";
import type { EditorMetaUpdate, EditorSession } from "../lib/types";

type Props = {
  editor: EditorSession;
  keys: string[];
  disabled?: boolean;
  onChange: (next: EditorSession) => void;
  onSave: () => Promise<void>;
  onCancel: () => void;
  onBodyDirtyChange?: (dirty: boolean) => void;
};

export function SongEditor({
  editor,
  keys,
  disabled,
  onChange,
  onSave,
  onCancel,
  onBodyDirtyChange,
}: Props) {
  const [title, setTitle] = useState(editor.title);
  const [artist, setArtist] = useState(editor.artist ?? "");
  const [album, setAlbum] = useState(editor.album ?? "");
  const [originalKey, setOriginalKey] = useState(editor.originalKey ?? "");
  const [tempo, setTempo] = useState(editor.tempoBpm?.toString() ?? "");
  const [timeSignature, setTimeSignature] = useState(
    editor.timeSignature ?? "",
  );
  const [notes, setNotes] = useState(editor.notes ?? "");
  const [tags, setTags] = useState(editor.tags.join(", "));
  const [chart, setChart] = useState(editor.chartText);

  useEffect(() => {
    setTitle(editor.title);
    setArtist(editor.artist ?? "");
    setAlbum(editor.album ?? "");
    setOriginalKey(editor.originalKey ?? "");
    setTempo(editor.tempoBpm?.toString() ?? "");
    setTimeSignature(editor.timeSignature ?? "");
    setNotes(editor.notes ?? "");
    setTags(editor.tags.join(", "));
    setChart(editor.chartText);
    onBodyDirtyChange?.(false);
    // Rehydrate only when switching songs so typing isn't overwritten by parse round-trips.
    // eslint-disable-next-line react-hooks/exhaustive-deps -- songId is the intentional gate
  }, [editor.songId]);

  function currentMeta(): EditorMetaUpdate {
    const bpm = tempo.trim() === "" ? null : Number(tempo);
    return {
      title,
      artist: artist || null,
      album: album || null,
      originalKey: originalKey || null,
      tempoBpm: bpm !== null && Number.isFinite(bpm) ? bpm : null,
      timeSignature: timeSignature || null,
      notes: notes || null,
      tags: tags
        .split(",")
        .map((tag) => tag.trim())
        .filter(Boolean),
    };
  }

  async function run(action: () => Promise<EditorSession>): Promise<void> {
    onChange(await action());
  }

  async function flushMeta(): Promise<EditorSession> {
    return editorUpdateMeta(currentMeta());
  }

  function updateChart(value: string) {
    setChart(value);
    onBodyDirtyChange?.(value !== editor.chartText);
  }

  return (
    <article className="song-editor panel">
      <header className="editor-header">
        <h2>{editor.isNew ? "New song" : "Edit song"}</h2>
        <p className="hint">
          {editor.dirty || chart !== editor.chartText
            ? "Unsaved changes."
            : "Saved draft."}{" "}
          Type chords on one line and lyrics on the next. Spaces line chords up
          with the words.
        </p>
        <div className="editor-toolbar">
          <button
            type="button"
            className="primary-button"
            disabled={disabled}
            onClick={() =>
              void (async () => {
                onChange(await editorParseBody(chart));
                onChange(await flushMeta());
                await onSave();
                onBodyDirtyChange?.(false);
              })()
            }
          >
            {disabled ? "Saving…" : "Save song"}
          </button>
          <button
            type="button"
            className="text-button"
            disabled={disabled}
            onClick={onCancel}
          >
            Cancel
          </button>
        </div>
      </header>

      {editor.summaryMessage && (
        <div className="import-warning" role="status">
          <p>{editor.summaryMessage}</p>
          <ul>
            {editor.warnings.map((warning, index) => (
              <li key={`${warning.kind}-${warning.line}-${index}-${warning.message}`}>
                {warning.message}
              </li>
            ))}
          </ul>
        </div>
      )}

      <div className="details-form">
        <label className="field-label">
          Title
          <input
            value={title}
            disabled={disabled}
            onChange={(event) => setTitle(event.target.value)}
            onBlur={() => void run(flushMeta)}
          />
        </label>
        <label className="field-label">
          Artist
          <input
            value={artist}
            disabled={disabled}
            onChange={(event) => setArtist(event.target.value)}
            onBlur={() => void run(flushMeta)}
          />
        </label>
        <label className="field-label">
          Album
          <input
            value={album}
            disabled={disabled}
            onChange={(event) => setAlbum(event.target.value)}
            onBlur={() => void run(flushMeta)}
          />
        </label>
        <label className="field-label">
          Original key
          <select
            value={originalKey}
            disabled={disabled}
            onChange={(event) => {
              setOriginalKey(event.target.value);
              void run(async () =>
                editorUpdateMeta({
                  ...currentMeta(),
                  originalKey: event.target.value || null,
                }),
              );
            }}
          >
            <option value="">None</option>
            {keys.map((key) => (
              <option key={key} value={key}>
                {key}
              </option>
            ))}
          </select>
        </label>
        <label className="field-label">
          Tempo
          <input
            inputMode="numeric"
            value={tempo}
            disabled={disabled}
            placeholder="BPM"
            onChange={(event) => setTempo(event.target.value)}
            onBlur={() => void run(flushMeta)}
          />
        </label>
        <label className="field-label">
          Time signature
          <input
            value={timeSignature}
            disabled={disabled}
            placeholder="4/4"
            onChange={(event) => setTimeSignature(event.target.value)}
            onBlur={() => void run(flushMeta)}
          />
        </label>
        <label className="field-label">
          Notes
          <textarea
            rows={2}
            value={notes}
            disabled={disabled}
            onChange={(event) => setNotes(event.target.value)}
            onBlur={() => void run(flushMeta)}
          />
        </label>
        <label className="field-label">
          Tags
          <input
            value={tags}
            disabled={disabled}
            placeholder="comma, separated"
            onChange={(event) => setTags(event.target.value)}
            onBlur={() => void run(flushMeta)}
          />
        </label>
      </div>

      <label className="field-label editor-chart-label">
        Chart
        <textarea
          className="editor-chart"
          aria-label="Chord and lyric chart"
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          wrap="off"
          rows={18}
          value={chart}
          disabled={disabled}
          placeholder={"[Verse]\nG          C\nType lyrics under the chords"}
          onChange={(event) => updateChart(event.target.value)}
          onBlur={() =>
            void run(async () => {
              const next = await editorParseBody(chart);
              onBodyDirtyChange?.(false);
              return next;
            })
          }
        />
      </label>
      <p className="hint">
        Section headers look like [Verse], [Chorus 2], or [Bridge]. Chord-only
        lines (intros, riffs) are fine without lyrics underneath.
      </p>
    </article>
  );
}
