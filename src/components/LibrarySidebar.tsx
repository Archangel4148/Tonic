import type {
  LibraryList,
  LibrarySort,
  LibrarySongSummary,
  SetlistSummary,
} from "../lib/types";

export type LibraryTab = "songs" | "setlists";

type Props = {
  library: LibraryList | null;
  setlists: SetlistSummary[];
  tab: LibraryTab;
  activeId: string | null;
  activeSetlistId: string | null;
  search: string;
  favoritesOnly: boolean;
  artist: string;
  songKey: string;
  tag: string;
  sort: LibrarySort;
  disabled?: boolean;
  onTabChange: (tab: LibraryTab) => void;
  onSearchChange: (value: string) => void;
  onFavoritesOnlyChange: (value: boolean) => void;
  onArtistChange: (value: string) => void;
  onKeyChange: (value: string) => void;
  onTagChange: (value: string) => void;
  onSortChange: (value: LibrarySort) => void;
  onOpen: (id: string) => void;
  onToggleFavorite: (id: string) => void;
  onNewSong: () => void;
  onImport: () => void;
  onOpenSetlist: (id: string) => void;
  onNewSetlist: () => void;
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
      <div
        className={active ? "library-row library-row--active" : "library-row"}
      >
        <button
          type="button"
          className="library-star"
          aria-pressed={song.favorite}
          aria-label={
            song.favorite
              ? `Unfavorite ${song.title}`
              : `Favorite ${song.title}`
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
  setlists,
  tab,
  activeId,
  activeSetlistId,
  search,
  favoritesOnly,
  artist,
  songKey,
  tag,
  sort,
  disabled,
  onTabChange,
  onSearchChange,
  onFavoritesOnlyChange,
  onArtistChange,
  onKeyChange,
  onTagChange,
  onSortChange,
  onOpen,
  onToggleFavorite,
  onNewSong,
  onImport,
  onOpenSetlist,
  onNewSetlist,
}: Props) {
  const songs = library?.songs ?? [];
  const recents = library?.recents ?? [];

  return (
    <aside className="library-sidebar" aria-label="Song library">
      <div className="library-heading">
        <h2>Library</h2>
        {tab === "songs" ? (
          <div className="library-heading-actions">
            <button type="button" className="text-button" onClick={onImport}>
              Import
            </button>
            <button type="button" className="text-button" onClick={onNewSong}>
              New song
            </button>
          </div>
        ) : (
          <button type="button" className="text-button" onClick={onNewSetlist}>
            New setlist
          </button>
        )}
      </div>
      <div
        className="library-tabs"
        role="tablist"
        aria-label="Library sections"
      >
        <button
          type="button"
          role="tab"
          id="library-tab-songs"
          aria-controls="library-panel-songs"
          aria-selected={tab === "songs"}
          tabIndex={tab === "songs" ? 0 : -1}
          className={tab === "songs" ? "chip chip--active" : "chip"}
          onClick={() => onTabChange("songs")}
        >
          Songs
        </button>
        <button
          type="button"
          role="tab"
          id="library-tab-setlists"
          aria-controls="library-panel-setlists"
          aria-selected={tab === "setlists"}
          tabIndex={tab === "setlists" ? 0 : -1}
          className={tab === "setlists" ? "chip chip--active" : "chip"}
          onClick={() => onTabChange("setlists")}
        >
          Setlists
        </button>
      </div>

      {tab === "songs" ? (
        <div
          id="library-panel-songs"
          role="tabpanel"
          aria-labelledby="library-tab-songs"
        >
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
              <select
                value={tag}
                onChange={(event) => onTagChange(event.target.value)}
              >
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
                onChange={(event) =>
                  onSortChange(event.target.value as LibrarySort)
                }
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
              <p className="hint">
                {search || favoritesOnly || artist || songKey || tag
                  ? "No songs match these filters."
                  : "No songs yet — use Import to add a chart."}
              </p>
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
        </div>
      ) : (
        <div
          id="library-panel-setlists"
          role="tabpanel"
          aria-labelledby="library-tab-setlists"
        >
          <section className="library-section" aria-label="All setlists">
            <h3>
              {setlists.length === 1
                ? "1 setlist"
                : `${setlists.length} setlists`}
            </h3>
            {setlists.length === 0 ? (
              <p className="hint">Create a setlist for rehearsal or a gig.</p>
            ) : (
              <ul className="library-list">
                {setlists.map((setlist) => (
                  <li key={setlist.id}>
                    <div
                      className={
                        setlist.id === activeSetlistId
                          ? "library-row library-row--setlist library-row--active"
                          : "library-row library-row--setlist"
                      }
                    >
                      <button
                        type="button"
                        className="library-open"
                        disabled={disabled}
                        onClick={() => onOpenSetlist(setlist.id)}
                      >
                        <span className="library-title">{setlist.name}</span>
                        <span className="library-meta">
                          {setlist.songCount === 1
                            ? "1 song"
                            : `${setlist.songCount} songs`}
                          {setlist.eventDate ? ` · ${setlist.eventDate}` : ""}
                        </span>
                      </button>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </section>
        </div>
      )}
    </aside>
  );
}
