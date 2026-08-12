import { useEffect, useState } from "react";
import type { SongSession } from "../lib/types";

type Props = {
  session: SongSession;
  disabled?: boolean;
  onSave: (values: {
    title: string;
    artist: string;
    album: string;
    notes: string;
    tags: string;
  }) => void;
  onDuplicate: () => void;
  onDelete: () => void;
};

export function SongDetails({
  session,
  disabled,
  onSave,
  onDuplicate,
  onDelete,
}: Props) {
  const [title, setTitle] = useState(session.song.title);
  const [artist, setArtist] = useState(session.song.artist ?? "");
  const [album, setAlbum] = useState(session.song.album ?? "");
  const [notes, setNotes] = useState(session.song.notes ?? "");
  const [tags, setTags] = useState(session.tags.join(", "));

  useEffect(() => {
    setTitle(session.song.title);
    setArtist(session.song.artist ?? "");
    setAlbum(session.song.album ?? "");
    setNotes(session.song.notes ?? "");
    setTags(session.tags.join(", "));
  }, [session]);

  return (
    <details className="song-details panel">
      <summary>Details</summary>
      <form
        className="details-form"
        onSubmit={(event) => {
          event.preventDefault();
          onSave({ title, artist, album, notes, tags });
        }}
      >
        <label className="field-label">
          Title
          <input
            value={title}
            required
            disabled={disabled}
            onChange={(event) => setTitle(event.target.value)}
          />
        </label>
        <label className="field-label">
          Artist
          <input
            value={artist}
            disabled={disabled}
            onChange={(event) => setArtist(event.target.value)}
          />
        </label>
        <label className="field-label">
          Album
          <input
            value={album}
            disabled={disabled}
            onChange={(event) => setAlbum(event.target.value)}
          />
        </label>
        <label className="field-label">
          Notes
          <textarea
            value={notes}
            rows={3}
            disabled={disabled}
            onChange={(event) => setNotes(event.target.value)}
          />
        </label>
        <label className="field-label">
          Tags
          <input
            value={tags}
            placeholder="comma, separated"
            disabled={disabled}
            onChange={(event) => setTags(event.target.value)}
          />
        </label>
        <div className="details-actions">
          <button type="submit" className="primary-button" disabled={disabled}>
            Save details
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
            Delete
          </button>
        </div>
      </form>
    </details>
  );
}
