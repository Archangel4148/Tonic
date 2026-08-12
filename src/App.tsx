import { useEffect, useMemo, useState } from "react";
import { EngineStatus } from "./components/EngineStatus";
import { ImportPanel } from "./components/ImportPanel";
import { LibrarySidebar } from "./components/LibrarySidebar";
import { SongDetails } from "./components/SongDetails";
import { SongEditor } from "./components/SongEditor";
import { SongViewer } from "./components/SongViewer";
import { TransposeBar } from "./components/TransposeBar";
import { TypeScaleControls } from "./components/TypeScaleControls";
import {
  beginEdit,
  cancelEdit,
  clearSong,
  createSong,
  deleteLibrarySong,
  duplicateLibrarySong,
  getAppInfo,
  getCurrentSong,
  getEditorState,
  importSong,
  listLibrary,
  openLibrarySong,
  resetPerformanceKey,
  saveEdit,
  setPerformanceKey,
  toggleFavorite,
  transposeSong,
  updateMetadata,
} from "./lib/tauri";
import {
  applyTheme,
  applyTypeScale,
  loadTheme,
  loadTypeScale,
  persistTypeScale,
} from "./lib/theme";
import type {
  AppInfo,
  ImportFormat,
  LibraryList,
  LibrarySort,
  EditorSession,
  SongSession,
  ThemePreference,
  TypeScale,
} from "./lib/types";
import "./App.css";

type BootState =
  | { status: "loading" }
  | { status: "ready"; info: AppInfo }
  | { status: "error"; message: string };

function App() {
  const [boot, setBoot] = useState<BootState>({ status: "loading" });
  const [session, setSession] = useState<SongSession | null>(null);
  const [editor, setEditor] = useState<EditorSession | null>(null);
  const [library, setLibrary] = useState<LibraryList | null>(null);
  const [search, setSearch] = useState("");
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [artistFilter, setArtistFilter] = useState("");
  const [keyFilter, setKeyFilter] = useState("");
  const [tagFilter, setTagFilter] = useState("");
  const [sort, setSort] = useState<LibrarySort>("title");
  const [importText, setImportText] = useState("");
  const [importFormat, setImportFormat] = useState<ImportFormat>("auto");
  const [importOpen, setImportOpen] = useState(true);
  const [busy, setBusy] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [theme, setTheme] = useState<ThemePreference>(() =>
    typeof localStorage === "undefined" ? "dark" : loadTheme(),
  );
  const [typeScale, setTypeScale] = useState<TypeScale>(() =>
    typeof localStorage === "undefined"
      ? { lyric: 1.2, chord: 1.05, section: 0.82 }
      : loadTypeScale(),
  );

  const keys = useMemo(
    () => (boot.status === "ready" ? boot.info.performanceKeys : []),
    [boot],
  );

  const query = useMemo(
    () => ({
      search: search || null,
      artist: artistFilter || null,
      key: keyFilter || null,
      favoritesOnly,
      tag: tagFilter || null,
      sort,
    }),
    [search, artistFilter, keyFilter, favoritesOnly, tagFilter, sort],
  );

  async function refreshLibrary(): Promise<void> {
    setLibrary(await listLibrary(query));
  }

  useEffect(() => {
    let cancelled = false;

    Promise.all([getAppInfo(), getCurrentSong(), listLibrary({}), getEditorState()])
      .then(([info, current, songs, openEditor]) => {
        if (cancelled) {
          return;
        }
        setBoot({ status: "ready", info });
        setLibrary(songs);
        if (openEditor) {
          setEditor(openEditor);
          setImportOpen(false);
        }
        if (current) {
          setSession(current);
          if (!openEditor) {
            setImportOpen(false);
          }
        }
      })
      .catch((error: unknown) => {
        const message =
          error instanceof Error
            ? error.message
            : "Could not reach the Tonic engine.";
        if (!cancelled) {
          setBoot({ status: "error", message });
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (boot.status !== "ready") {
      return;
    }
    let cancelled = false;
    listLibrary(query)
      .then((songs) => {
        if (!cancelled) {
          setLibrary(songs);
        }
      })
      .catch(() => {
        /* keep last library snapshot */
      });
    return () => {
      cancelled = true;
    };
  }, [boot.status, query]);

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  useEffect(() => {
    applyTypeScale(typeScale);
    persistTypeScale(typeScale);
  }, [typeScale]);

  async function confirmLeaveEditor(): Promise<boolean> {
    if (!editor) {
      return true;
    }
    if (
      editor.dirty &&
      !window.confirm("Discard unsaved editor changes?")
    ) {
      return false;
    }
    const remaining = await cancelEdit();
    setEditor(null);
    setSession(remaining);
    return true;
  }

  async function runAction(action: () => Promise<SongSession>): Promise<void> {
    setBusy(true);
    setActionError(null);
    try {
      const next = await action();
      setSession(next);
      await refreshLibrary();
    } catch (error: unknown) {
      setActionError(
        error instanceof Error ? error.message : "Something went wrong.",
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="brand">
          <p className="eyebrow">Songbook</p>
          <h1>Tonic</h1>
        </div>
        <div className="header-tools">
          <div className="theme-toggle" role="group" aria-label="Theme">
            {(["dark", "light", "system"] as const).map((option) => (
              <button
                key={option}
                type="button"
                className={theme === option ? "chip chip--active" : "chip"}
                onClick={() => setTheme(option)}
              >
                {option === "dark"
                  ? "Dark"
                  : option === "light"
                    ? "Light"
                    : "System"}
              </button>
            ))}
          </div>
          {boot.status === "ready" && <EngineStatus info={boot.info} />}
        </div>
      </header>

      <div className="app-workspace">
        {boot.status === "ready" && (
          <LibrarySidebar
            library={library}
            activeId={editor?.songId ?? session?.song.id ?? null}
            search={search}
            favoritesOnly={favoritesOnly}
            artist={artistFilter}
            songKey={keyFilter}
            tag={tagFilter}
            sort={sort}
            disabled={busy}
            onSearchChange={setSearch}
            onFavoritesOnlyChange={setFavoritesOnly}
            onArtistChange={setArtistFilter}
            onKeyChange={setKeyFilter}
            onTagChange={setTagFilter}
            onSortChange={setSort}
            onNewSong={() =>
              void (async () => {
                if (!(await confirmLeaveEditor())) {
                  return;
                }
                setBusy(true);
                setActionError(null);
                try {
                  const next = await createSong();
                  setEditor(next);
                  setImportOpen(false);
                } catch (error: unknown) {
                  setActionError(
                    error instanceof Error
                      ? error.message
                      : "Something went wrong.",
                  );
                } finally {
                  setBusy(false);
                }
              })()
            }
            onOpen={(id) =>
              void (async () => {
                if (!(await confirmLeaveEditor())) {
                  return;
                }
                await runAction(async () => {
                  const next = await openLibrarySong(id);
                  setEditor(null);
                  setImportOpen(false);
                  return next;
                });
              })()
            }
            onToggleFavorite={(id) =>
              void (async () => {
                setBusy(true);
                setActionError(null);
                try {
                  const next = await toggleFavorite(id);
                  if (next) {
                    setSession(next);
                  }
                  await refreshLibrary();
                } catch (error: unknown) {
                  setActionError(
                    error instanceof Error
                      ? error.message
                      : "Something went wrong.",
                  );
                } finally {
                  setBusy(false);
                }
              })()
            }
          />
        )}

        <main>
          {boot.status === "loading" && (
            <p role="status">Connecting to the local engine…</p>
          )}

          {boot.status === "error" && (
            <div className="panel" role="alert">
              <h2>Engine unavailable</h2>
              <p>{boot.message}</p>
              <p className="hint">
                Run the desktop app with <code>npm run tauri dev</code> so the UI
                can talk to the Rust engine.
              </p>
            </div>
          )}

          {boot.status === "ready" && (
            <>
              <div className="toolbar">
                <button
                  type="button"
                  className="text-button"
                  onClick={() => setImportOpen((open) => !open)}
                >
                  {importOpen
                    ? "Hide import"
                    : session
                      ? "Import another"
                      : "Import"}
                </button>
                {session && !editor && (
                  <button
                    type="button"
                    className="text-button"
                    onClick={() =>
                      void (async () => {
                        setBusy(true);
                        setActionError(null);
                        try {
                          const next = await beginEdit(session.song.id);
                          setEditor(next);
                          setImportOpen(false);
                        } catch (error: unknown) {
                          setActionError(
                            error instanceof Error
                              ? error.message
                              : "Something went wrong.",
                          );
                        } finally {
                          setBusy(false);
                        }
                      })()
                    }
                  >
                    Edit song
                  </button>
                )}
                {(session || editor) && (
                  <button
                    type="button"
                    className="text-button"
                    onClick={async () => {
                      if (!(await confirmLeaveEditor())) {
                        return;
                      }
                      await clearSong();
                      setSession(null);
                      setEditor(null);
                      setImportOpen(false);
                      setActionError(null);
                    }}
                  >
                    Close song
                  </button>
                )}
                {session && !editor && (
                  <TransposeBar
                    originalKey={session.song.originalKey}
                    performanceKey={session.song.performanceKey}
                    semitoneOffset={session.semitoneOffset}
                    keys={keys}
                    disabled={busy}
                    onTranspose={(semitones) =>
                      void runAction(() => transposeSong(semitones))
                    }
                    onSelectKey={(key) =>
                      void runAction(() => setPerformanceKey(key))
                    }
                    onReset={() => void runAction(() => resetPerformanceKey())}
                  />
                )}
                <TypeScaleControls scale={typeScale} onChange={setTypeScale} />
              </div>

              {actionError && (
                <p className="panel action-error" role="alert">
                  {actionError}
                </p>
              )}

              {importOpen && !editor && (
                <ImportPanel
                  text={importText}
                  format={importFormat}
                  busy={busy}
                  onTextChange={setImportText}
                  onFormatChange={setImportFormat}
                  onImport={(text, format) =>
                    void (async () => {
                      if (!(await confirmLeaveEditor())) {
                        return;
                      }
                      await runAction(async () => {
                        setImportText(text);
                        setImportFormat(format);
                        const next = await importSong(text, format);
                        setEditor(null);
                        setImportOpen(false);
                        return next;
                      });
                    })()
                  }
                />
              )}

              {editor ? (
                <SongEditor
                  editor={editor}
                  keys={keys}
                  disabled={busy}
                  onChange={setEditor}
                  onSave={async () => {
                    setBusy(true);
                    setActionError(null);
                    try {
                      const result = await saveEdit();
                      setEditor(result.editor);
                      setSession(result.session);
                      await refreshLibrary();
                    } catch (error: unknown) {
                      setActionError(
                        error instanceof Error
                          ? error.message
                          : "Something went wrong.",
                      );
                      throw error;
                    } finally {
                      setBusy(false);
                    }
                  }}
                  onCancel={() =>
                    void (async () => {
                      if (
                        editor.dirty &&
                        !window.confirm("Discard unsaved editor changes?")
                      ) {
                        return;
                      }
                      const remaining = await cancelEdit();
                      setEditor(null);
                      setSession(remaining);
                    })()
                  }
                />
              ) : session ? (
                <>
                  <SongViewer session={session} />
                  <SongDetails
                    session={session}
                    disabled={busy}
                    onSave={(values) =>
                      void runAction(() =>
                        updateMetadata({
                          title: values.title,
                          artist: values.artist || null,
                          album: values.album || null,
                          notes: values.notes || null,
                          tags: values.tags
                            .split(",")
                            .map((tag) => tag.trim())
                            .filter(Boolean),
                        }),
                      )
                    }
                    onDuplicate={() =>
                      void (async () => {
                        if (!(await confirmLeaveEditor())) {
                          return;
                        }
                        await runAction(() =>
                          duplicateLibrarySong(session.song.id),
                        );
                      })()
                    }
                    onDelete={() => {
                      if (
                        !window.confirm(
                          `Delete “${session.song.title}” from this library?`,
                        )
                      ) {
                        return;
                      }
                      void (async () => {
                        setBusy(true);
                        setActionError(null);
                        try {
                          const remaining = await deleteLibrarySong(
                            session.song.id,
                          );
                          setSession(remaining);
                          await refreshLibrary();
                        } catch (error: unknown) {
                          setActionError(
                            error instanceof Error
                              ? error.message
                              : "Something went wrong.",
                          );
                        } finally {
                          setBusy(false);
                        }
                      })();
                    }}
                  />
                </>
              ) : (
                !importOpen && (
                  <p className="hint empty-hint">
                    Open a song from the library or import a chart.
                  </p>
                )
              )}
            </>
          )}
        </main>
      </div>

      <footer>
        <p>Songs are saved locally on this device and stay available offline.</p>
      </footer>
    </div>
  );
}

export default App;
