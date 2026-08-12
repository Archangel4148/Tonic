import { ChartLine } from "./ChartLine";
import type { SongSession } from "../lib/types";

type Props = {
  session: SongSession;
  disabled?: boolean;
  onCapoChange?: (fret: number | null) => void;
};

export function SongViewer({ session, disabled, onCapoChange }: Props) {
  const {
    song,
    warnings,
    summaryMessage,
    semitoneOffset,
    favorite,
    tags,
    setlist,
  } = session;

  return (
    <article className="song-viewer" aria-labelledby="song-title">
      {setlist && (
        <aside className="setlist-banner" aria-label="Setlist context">
          <p>
            <strong>{setlist.setlistName}</strong>
            {" · "}
            {setlist.index + 1} of {setlist.total}
            {setlist.playedKey && (
              <>
                {" · "}
                Played <strong>{setlist.playedKey}</strong>
              </>
            )}
          </p>
          {onCapoChange && (
            <label className="field-label">
              Capo
              <select
                value={setlist.capoFret == null ? "" : String(setlist.capoFret)}
                disabled={disabled}
                onChange={(event) =>
                  onCapoChange(
                    event.target.value === ""
                      ? null
                      : Number(event.target.value),
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
          )}
          {setlist.entryNotes && (
            <p className="song-notes">{setlist.entryNotes}</p>
          )}
        </aside>
      )}
      <header className="song-header">
        <h2 id="song-title">
          {favorite ? "★ " : ""}
          {song.title}
        </h2>
        {song.artist && <p className="song-artist">{song.artist}</p>}
        {tags.length > 0 && (
          <p className="song-tags">
            {tags.map((tag) => (
              <span key={tag} className="tag-chip">
                {tag}
              </span>
            ))}
          </p>
        )}
        <p className="song-meta">
          {song.originalKey && (
            <span>
              Original <strong>{song.originalKey}</strong>
            </span>
          )}
          {song.performanceKey && (
            <span>
              Now <strong>{song.performanceKey}</strong>
              {semitoneOffset !== 0 && (
                <>
                  {" "}
                  ({semitoneOffset > 0 ? "+" : ""}
                  {semitoneOffset})
                </>
              )}
            </span>
          )}
          {song.timeSignature && <span>{song.timeSignature}</span>}
          {song.tempoBpm && <span>{song.tempoBpm} BPM</span>}
        </p>
        {song.notes && <p className="song-notes">{song.notes}</p>}
      </header>

      {summaryMessage && (
        <div className="import-warning" role="status">
          <p>{summaryMessage}</p>
          <ul>
            {warnings.map((warning) => (
              <li key={`${warning.kind}-${warning.line}-${warning.message}`}>
                {warning.line != null ? `Line ${warning.line}: ` : ""}
                {warning.message}
              </li>
            ))}
          </ul>
        </div>
      )}

      <div className="song-body">
        {song.sections.map((section, sectionIndex) => (
          <section
            key={`${section.label}-${sectionIndex}`}
            className="song-section"
            aria-label={section.label}
          >
            <h3 className="section-label">{section.label}</h3>
            {section.lines.map((line, lineIndex) => (
              <ChartLine key={`${section.label}-${lineIndex}`} line={line} />
            ))}
          </section>
        ))}
      </div>
    </article>
  );
}
