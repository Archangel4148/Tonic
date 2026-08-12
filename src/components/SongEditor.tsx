import { useEffect, useState } from "react";
import {
  editorAddLine,
  editorAddSection,
  editorMoveSection,
  editorParseBody,
  editorRemoveLine,
  editorRemoveSection,
  editorReplaceChord,
  editorSetAnnotation,
  editorSetLyrics,
  editorSetSectionLabel,
  editorTagChord,
  editorUntagChord,
  editorUpdateMeta,
} from "../lib/tauri";
import type {
  EditorMetaUpdate,
  EditorSession,
  ImportFormat,
} from "../lib/types";

type Props = {
  editor: EditorSession;
  keys: string[];
  disabled?: boolean;
  onChange: (next: EditorSession) => void;
  onSave: () => Promise<void>;
  onCancel: () => void;
};

const SECTION_KINDS = [
  { value: "intro", label: "Intro" },
  { value: "verse", label: "Verse" },
  { value: "preChorus", label: "Pre-Chorus" },
  { value: "chorus", label: "Chorus" },
  { value: "bridge", label: "Bridge" },
  { value: "instrumental", label: "Instrumental" },
  { value: "solo", label: "Solo" },
  { value: "outro", label: "Outro" },
  { value: "custom", label: "Custom" },
] as const;

export function SongEditor({
  editor,
  keys,
  disabled,
  onChange,
  onSave,
  onCancel,
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
  const [pasteText, setPasteText] = useState("");
  const [pasteFormat, setPasteFormat] = useState<ImportFormat>("auto");
  const [newSectionKind, setNewSectionKind] = useState("verse");
  const [tagSymbol, setTagSymbol] = useState<Record<string, string>>({});
  const [carets, setCarets] = useState<Record<string, number>>({});
  const [lyricsDraft, setLyricsDraft] = useState<Record<string, string>>({});

  useEffect(() => {
    setTitle(editor.title);
    setArtist(editor.artist ?? "");
    setAlbum(editor.album ?? "");
    setOriginalKey(editor.originalKey ?? "");
    setTempo(editor.tempoBpm?.toString() ?? "");
    setTimeSignature(editor.timeSignature ?? "");
    setNotes(editor.notes ?? "");
    setTags(editor.tags.join(", "));
    const nextLyrics: Record<string, string> = {};
    editor.sections.forEach((section, sectionIndex) => {
      section.lines.forEach((line, lineIndex) => {
        nextLyrics[`${sectionIndex}:${lineIndex}`] = line.lyrics;
      });
    });
    setLyricsDraft(nextLyrics);
  }, [editor]);

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

  return (
    <article className="song-editor panel">
      <header className="editor-header">
        <h2>{editor.isNew ? "New song" : "Edit song"}</h2>
        <p className="hint">
          {editor.dirty ? "Unsaved changes." : "Saved draft."} Chords are parsed
          by the engine, not the UI.
        </p>
        <div className="editor-toolbar">
          <button
            type="button"
            className="primary-button"
            disabled={disabled}
            onClick={() =>
              void (async () => {
                onChange(await flushMeta());
                await onSave();
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

      {editor.sections.map((section, sectionIndex) => (
        <section
          key={`${section.kind}-${sectionIndex}`}
          className="editor-section"
          aria-label={`Edit ${section.label}`}
        >
          <div className="editor-section-bar">
            <label className="field-label">
              Section
              <select
                value={section.kind}
                disabled={disabled}
                onChange={(event) =>
                  void run(() =>
                    editorSetSectionLabel(sectionIndex, {
                      kind: event.target.value,
                      number: section.number,
                      customName: section.customName,
                    }),
                  )
                }
              >
                {SECTION_KINDS.map((kind) => (
                  <option key={kind.value} value={kind.value}>
                    {kind.label}
                  </option>
                ))}
              </select>
            </label>
            {(section.kind === "verse" || section.kind === "chorus") && (
              <label className="field-label">
                Number
                <input
                  inputMode="numeric"
                  value={section.number ?? ""}
                  disabled={disabled}
                  onChange={(event) => {
                    const value = event.target.value.trim();
                    const number = value === "" ? null : Number(value);
                    void run(() =>
                      editorSetSectionLabel(sectionIndex, {
                        kind: section.kind,
                        number:
                          number !== null && Number.isFinite(number)
                            ? number
                            : null,
                        customName: null,
                      }),
                    );
                  }}
                />
              </label>
            )}
            {section.kind === "custom" && (
              <label className="field-label">
                Name
                <input
                  value={section.customName ?? ""}
                  disabled={disabled}
                  onBlur={(event) =>
                    void run(() =>
                      editorSetSectionLabel(sectionIndex, {
                        kind: "custom",
                        number: null,
                        customName: event.target.value || "Custom",
                      }),
                    )
                  }
                  onChange={(event) =>
                    void run(() =>
                      editorSetSectionLabel(sectionIndex, {
                        kind: "custom",
                        number: null,
                        customName: event.target.value || "Custom",
                      }),
                    )
                  }
                />
              </label>
            )}
            <button
              type="button"
              className="text-button"
              disabled={disabled || sectionIndex === 0}
              onClick={() =>
                void run(() =>
                  editorMoveSection(sectionIndex, sectionIndex - 1),
                )
              }
            >
              Up
            </button>
            <button
              type="button"
              className="text-button"
              disabled={disabled || sectionIndex === editor.sections.length - 1}
              onClick={() =>
                void run(() =>
                  editorMoveSection(sectionIndex, sectionIndex + 1),
                )
              }
            >
              Down
            </button>
            <button
              type="button"
              className="text-button"
              disabled={disabled || editor.sections.length <= 1}
              onClick={() => void run(() => editorRemoveSection(sectionIndex))}
            >
              Remove section
            </button>
          </div>

          {section.lines.map((line, lineIndex) => {
            const key = `${sectionIndex}:${lineIndex}`;
            return (
              <div key={key} className="editor-line">
                <div className="editor-chords" aria-label="Chord tags">
                  {line.chords.map((chord, chordIndex) => (
                    <span
                      key={`${chord.symbol}-${chord.lyricIndex}-${chordIndex}`}
                      className={`chord-tag chord-tag--${chord.status}`}
                    >
                      <input
                        aria-label={`Chord ${chordIndex + 1}`}
                        defaultValue={chord.symbol}
                        disabled={disabled}
                        onBlur={(event) => {
                          const next = event.target.value.trim();
                          if (!next || next === chord.symbol) {
                            return;
                          }
                          void run(() =>
                            editorReplaceChord(
                              sectionIndex,
                              lineIndex,
                              chordIndex,
                              next,
                            ),
                          );
                        }}
                      />
                      <span className="chord-index">@{chord.lyricIndex}</span>
                      <button
                        type="button"
                        className="icon-button icon-button--small"
                        aria-label={`Remove chord ${chord.symbol}`}
                        disabled={disabled}
                        onClick={() =>
                          void run(() =>
                            editorUntagChord(
                              sectionIndex,
                              lineIndex,
                              chordIndex,
                            ),
                          )
                        }
                      >
                        ×
                      </button>
                    </span>
                  ))}
                </div>
                <label className="field-label">
                  Lyrics
                  <input
                    value={lyricsDraft[key] ?? line.lyrics}
                    disabled={disabled}
                    onSelect={(event) =>
                      setCarets((current) => ({
                        ...current,
                        [key]: event.currentTarget.selectionStart ?? 0,
                      }))
                    }
                    onChange={(event) =>
                      setLyricsDraft((current) => ({
                        ...current,
                        [key]: event.target.value,
                      }))
                    }
                    onBlur={(event) =>
                      void run(() =>
                        editorSetLyrics(
                          sectionIndex,
                          lineIndex,
                          event.target.value,
                        ),
                      )
                    }
                  />
                </label>
                <div className="editor-line-tools">
                  <input
                    aria-label={`Chord symbol for ${section.label} line ${lineIndex + 1}`}
                    placeholder="G"
                    value={tagSymbol[key] ?? ""}
                    disabled={disabled}
                    onChange={(event) =>
                      setTagSymbol((current) => ({
                        ...current,
                        [key]: event.target.value,
                      }))
                    }
                  />
                  <button
                    type="button"
                    className="text-button"
                    disabled={disabled || !(tagSymbol[key] ?? "").trim()}
                    onClick={() =>
                      void run(async () => {
                        const next = await editorTagChord(
                          sectionIndex,
                          lineIndex,
                          carets[key] ?? 0,
                          (tagSymbol[key] ?? "").trim(),
                        );
                        setTagSymbol((current) => ({ ...current, [key]: "" }));
                        return next;
                      })
                    }
                  >
                    Tag chord at caret
                  </button>
                  <input
                    aria-label={`Annotation for ${section.label} line ${lineIndex + 1}`}
                    placeholder="Annotation"
                    defaultValue={line.annotation ?? ""}
                    disabled={disabled}
                    onBlur={(event) =>
                      void run(() =>
                        editorSetAnnotation(
                          sectionIndex,
                          lineIndex,
                          event.target.value.trim() || null,
                        ),
                      )
                    }
                  />
                  <button
                    type="button"
                    className="text-button"
                    disabled={disabled}
                    onClick={() => void run(() => editorAddLine(sectionIndex))}
                  >
                    Add line
                  </button>
                  <button
                    type="button"
                    className="text-button"
                    disabled={disabled || section.lines.length <= 1}
                    onClick={() =>
                      void run(() => editorRemoveLine(sectionIndex, lineIndex))
                    }
                  >
                    Remove line
                  </button>
                </div>
              </div>
            );
          })}
        </section>
      ))}

      <div className="editor-toolbar">
        <label className="field-label">
          Add section
          <select
            value={newSectionKind}
            disabled={disabled}
            onChange={(event) => setNewSectionKind(event.target.value)}
          >
            {SECTION_KINDS.map((kind) => (
              <option key={kind.value} value={kind.value}>
                {kind.label}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          className="text-button"
          disabled={disabled}
          onClick={() =>
            void run(() =>
              editorAddSection({
                kind: newSectionKind,
                number: null,
                customName: newSectionKind === "custom" ? "Custom" : null,
              }),
            )
          }
        >
          Add section
        </button>
      </div>

      <details className="editor-paste">
        <summary>Paste chart to replace body</summary>
        <p className="hint">
          Parser correction: paste ChordPro or chord-over-lyrics. Metadata
          already filled in is kept.
        </p>
        <label className="field-label">
          Format
          <select
            value={pasteFormat}
            onChange={(event) =>
              setPasteFormat(event.target.value as ImportFormat)
            }
          >
            <option value="auto">Auto</option>
            <option value="chordPro">ChordPro</option>
            <option value="plainText">Plain text</option>
          </select>
        </label>
        <label className="field-label">
          Chart text
          <textarea
            value={pasteText}
            rows={6}
            onChange={(event) => setPasteText(event.target.value)}
          />
        </label>
        <button
          type="button"
          className="text-button"
          disabled={disabled || !pasteText.trim()}
          onClick={() =>
            void run(async () => {
              const next = await editorParseBody(pasteText, pasteFormat);
              setPasteText("");
              return next;
            })
          }
        >
          Replace chart
        </button>
      </details>
    </article>
  );
}
