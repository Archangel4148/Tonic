import type { LibraryList, LibrarySort, LibrarySongSummary } from "../lib/types";

type Props = {
  library: LibraryList | null;
  activeId: string | null;
  search: string;
  favoritesOnly: boolean;
  artist: string;
  songKey: string;
  tag: string;
  sort: LibrarySort;
  disabled?: boolean;
  onSearchChange: (value: string) => void;
  onFavoritesOnlyChange: (value: boolean) => void;
  onArtistChange: (value: string) => void;
  onKeyChange: (value: string) => void;
  onTagChange: (value: string) => void;
  onSortChange: (value: LibrarySort) => void;
  onOpen: (id: string) => void;
  onToggleFavorite: (id: string) => void;
  onNewSong: () => void;
};

function SongRow({
  song,
  active,
  disabled,
  onOpen,
  onToggleFavorite,
}: {
  song: LibrarySongSummary;
  active: boolean;
  disabled?: boolean;
  onOpen: (id: string) => void;
  onToggleFavorite: (id: string) => void;
}) {
  return (
    <li>
      <div className={active ? "library-row library-row--active" : "library-row"}>
        <button
          type="button"
          className="library-star"
          aria-pressed={song.favorite}
          aria-label={
            song.favorite ? `Unfavorite ${song.title}` : `Favorite ${song.title}`
          }
          disabled={disabled}
          onClick={() => onToggleFavorite(song.id)}
        >
          {song.favorite ? "★" : "☆"}
        </button>
        <button
          type="button"
          className="library-open"
          disabled={disabled}
          onClick={() => onOpen(song.id)}
        >
          <span className="library-title">{song.title}</span>
          <span className="library-meta">
            {song.artist ?? "Unknown artist"}
            {song.performanceKey ? ` · ${song.performanceKey}` : ""}
          </span>
        </button>
      </div>
    </li>
  );
}

export function LibrarySidebar({
  library,
  activeId,
  search,
  favoritesOnly,
  artist,
  songKey,
  tag,
  sort,
  disabled,
  onSearchChange,
  onFavoritesOnlyChange,
  onArtistChange,
  onKeyChange,
  onTagChange,
  onSortChange,
  onOpen,
  onToggleFavorite,
  onNewSong,
}: Props) {
  const songs = library?.songs ?? [];
  const recents = library?.recents ?? [];

  return (
    <aside className="library-sidebar" aria-label="Song library">
      <div className="library-heading">
        <h2>Library</h2>
        <button type="button" className="text-button" onClick={onNewSong}>
          New song
        </button>
      </div>
      <label className="field-label">
        Search
        <input
          type="search"
          value={search}
          placeholder="Title, artist, lyrics, tags"
          onChange={(event) => onSearchChange(event.target.value)}
        />
      </label>
      <div className="library-filters">
        <button
          type="button"
          className={favoritesOnly ? "chip chip--active" : "chip"}
          aria-pressed={favoritesOnly}
          onClick={() => onFavoritesOnlyChange(!favoritesOnly)}
        >
          Favorites
        </button>
        <label className="field-label">
          Artist
          <select
            value={artist}
            onChange={(event) => onArtistChange(event.target.value)}
          >
            <option value="">All artists</option>
            {(library?.artists ?? []).map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </label>
        <label className="field-label">
          Key
          <select
            value={songKey}
            onChange={(event) => onKeyChange(event.target.value)}
          >
            <option value="">All keys</option>
            {(library?.keys ?? []).map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </label>
        <label className="field-label">
          Tag
          <select value={tag} onChange={(event) => onTagChange(event.target.value)}>
            <option value="">All tags</option>
            {(library?.tags ?? []).map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </label>
        <label className="field-label">
          Sort
          <select
            value={sort}
            onChange={(event) => onSortChange(event.target.value as LibrarySort)}
          >
            <option value="title">Title</option>
            <option value="artist">Artist</option>
            <option value="recentOpened">Recently opened</option>
            <option value="recentModified">Recently modified</option>
          </select>
        </label>
      </div>

      {recents.length > 0 && sort !== "recentOpened" && (
        <section className="library-section" aria-label="Recent songs">
          <h3>Recent</h3>
          <ul className="library-list">
            {recents.map((song) => (
              <SongRow
                key={`recent-${song.id}`}
                song={song}
                active={song.id === activeId}
                disabled={disabled}
                onOpen={onOpen}
                onToggleFavorite={onToggleFavorite}
              />
            ))}
          </ul>
        </section>
      )}

      <section className="library-section" aria-label="All songs">
        <h3>{songs.length === 1 ? "1 song" : `${songs.length} songs`}</h3>
        {songs.length === 0 ? (
          <p className="hint">Import a chart to start your songbook.</p>
        ) : (
          <ul className="library-list">
            {songs.map((song) => (
              <SongRow
                key={song.id}
                song={song}
                active={song.id === activeId}
                disabled={disabled}
                onOpen={onOpen}
                onToggleFavorite={onToggleFavorite}
              />
            ))}
          </ul>
        )}
      </section>
    </aside>
  );
}
