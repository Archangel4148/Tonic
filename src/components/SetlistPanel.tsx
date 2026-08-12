import { useEffect, useState } from "react";
import type { LibrarySongSummary, Setlist } from "../lib/types";

type Props = {
  setlist: Setlist;
  songs: LibrarySongSummary[];
  keys: string[];
  activeEntryId: string | null;
  disabled?: boolean;
  onRename: (name: string, notes: string, eventDate: string) => void;
  onAddSong: (songId: string) => void;
  onRemoveEntry: (entryId: string) => void;
  onMoveEntry: (from: number, to: number) => void;
  onOpenEntry: (entryId: string) => void;
  onUpdateEntry: (
    entryId: string,
    performanceKey: string | null,
    capoFret: number | null,
    notes: string | null,
  ) => void;
  onDuplicate: () => void;
  onDelete: () => void;
  onPerform: () => void;
};

export function SetlistPanel({
  setlist,
  songs,
  keys,
  activeEntryId,
  disabled,
  onRename,
  onAddSong,
  onRemoveEntry,
  onMoveEntry,
  onOpenEntry,
  onUpdateEntry,
  onDuplicate,
  onDelete,
  onPerform,
}: Props) {
  const [name, setName] = useState(setlist.name);
  const [notes, setNotes] = useState(setlist.notes ?? "");
  const [eventDate, setEventDate] = useState(setlist.eventDate ?? "");
  const [addSongId, setAddSongId] = useState(songs[0]?.id ?? "");

  useEffect(() => {
    setName(setlist.name);
    setNotes(setlist.notes ?? "");
    setEventDate(setlist.eventDate ?? "");
  }, [setlist.id, setlist.name, setlist.notes, setlist.eventDate]);

  useEffect(() => {
    if (!addSongId && songs[0]) {
      setAddSongId(songs[0].id);
    }
  }, [songs, addSongId]);

  return (
    <article className="setlist-panel panel">
      <header className="setlist-header">
        <h2>Setlist</h2>
        <div className="editor-toolbar">
          <button
            type="button"
            className="text-button"
            disabled={
              disabled || !setlist.entries.some((entry) => !entry.missing)
            }
            onClick={onPerform}
          >
            Perform set
          </button>
          <button
            type="button"
            className="text-button"
            disabled={disabled}
            onClick={onDuplicate}
          >
            Duplicate
          </button>
          <button
            type="button"
            className="text-button"
            disabled={disabled}
            onClick={onDelete}
          >
            Delete setlist
          </button>
        </div>
      </header>
      <div className="details-form">
        <label className="field-label">
          Name
          <input
            value={name}
            disabled={disabled}
            onChange={(event) => setName(event.target.value)}
            onBlur={() => onRename(name, notes, eventDate)}
          />
        </label>
        <label className="field-label">
          Event / date
          <input
            value={eventDate}
            disabled={disabled}
            placeholder="optional"
            onChange={(event) => setEventDate(event.target.value)}
            onBlur={() => onRename(name, notes, eventDate)}
          />
        </label>
        <label className="field-label">
          Notes
          <textarea
            rows={2}
            value={notes}
            disabled={disabled}
            onChange={(event) => setNotes(event.target.value)}
            onBlur={() => onRename(name, notes, eventDate)}
          />
        </label>
      </div>

      <div className="editor-toolbar">
        <label className="field-label">
          Add song
          <select
            value={addSongId}
            disabled={disabled || songs.length === 0}
            onChange={(event) => setAddSongId(event.target.value)}
          >
            {songs.length === 0 && (
              <option value="">No songs in library</option>
            )}
            {songs.map((song) => (
              <option key={song.id} value={song.id}>
                {song.title}
                {song.artist ? ` — ${song.artist}` : ""}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          className="text-button"
          disabled={disabled || !addSongId}
          onClick={() => onAddSong(addSongId)}
        >
          Add to setlist
        </button>
      </div>

      {setlist.entries.length === 0 ? (
        <p className="hint">
          Add songs. The same song can appear more than once.
        </p>
      ) : (
        <ol className="setlist-entries">
          {setlist.entries.map((entry, index) => (
            <li
              key={entry.id}
              className={
                entry.id === activeEntryId
                  ? "setlist-entry setlist-entry--active"
                  : "setlist-entry"
              }
            >
              <button
                type="button"
                className="setlist-open"
                disabled={disabled || entry.missing}
                onClick={() => onOpenEntry(entry.id)}
              >
                <strong>
                  {index + 1}. {entry.title}
                </strong>
                <span className="library-meta">
                  {entry.artist ??
                    (entry.missing ? "Missing" : "Unknown artist")}
                  {entry.performanceKey
                    ? ` · ${entry.performanceKey}`
                    : entry.songKey
                      ? ` · ${entry.songKey}`
                      : ""}
                  {entry.capoFret != null ? ` · capo ${entry.capoFret}` : ""}
                </span>
              </button>
              <div className="setlist-entry-tools">
                <label className="field-label">
                  Key
                  <select
                    value={entry.performanceKey ?? ""}
                    disabled={disabled || entry.missing}
                    onChange={(event) =>
                      onUpdateEntry(
                        entry.id,
                        event.target.value || null,
                        entry.capoFret,
                        entry.notes,
                      )
                    }
                  >
                    <option value="">Song default</option>
                    {keys.map((key) => (
                      <option key={key} value={key}>
                        {key}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="field-label">
                  Capo
                  <select
                    value={entry.capoFret == null ? "" : String(entry.capoFret)}
                    disabled={disabled || entry.missing}
                    onChange={(event) =>
                      onUpdateEntry(
                        entry.id,
                        entry.performanceKey,
                        event.target.value === ""
                          ? null
                          : Number(event.target.value),
                        entry.notes,
                      )
                    }
                  >
                    <option value="">None</option>
                    {Array.from({ length: 13 }, (_, fret) => (
                      <option key={fret} value={String(fret)}>
                        {fret}
                      </option>
                    ))}
                  </select>
                </label>
                <input
                  key={`${entry.id}-${entry.notes ?? ""}`}
                  aria-label={`Notes for ${entry.title}`}
                  placeholder="Entry notes"
                  defaultValue={entry.notes ?? ""}
                  disabled={disabled}
                  onBlur={(event) =>
                    onUpdateEntry(
                      entry.id,
                      entry.performanceKey,
                      entry.capoFret,
                      event.target.value.trim() || null,
                    )
                  }
                />
                <button
                  type="button"
                  className="text-button"
                  disabled={disabled || index === 0}
                  aria-label={`Move ${entry.title} up`}
                  onClick={() => onMoveEntry(index, index - 1)}
                >
                  Up
                </button>
                <button
                  type="button"
                  className="text-button"
                  disabled={disabled || index === setlist.entries.length - 1}
                  aria-label={`Move ${entry.title} down`}
                  onClick={() => onMoveEntry(index, index + 1)}
                >
                  Down
                </button>
                <button
                  type="button"
                  className="text-button text-button--danger"
                  disabled={disabled}
                  aria-label={`Remove ${entry.title} from setlist`}
                  onClick={() => onRemoveEntry(entry.id)}
                >
                  Remove
                </button>
              </div>
            </li>
          ))}
        </ol>
      )}
    </article>
  );
}
