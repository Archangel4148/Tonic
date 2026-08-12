import { useCallback, useEffect, useMemo, useState } from "react";
import { EngineStatus } from "./components/EngineStatus";
import { ImportPanel } from "./components/ImportPanel";
import { LibrarySidebar, type LibraryTab } from "./components/LibrarySidebar";
import { LiveMode } from "./components/LiveMode";
import { SetlistPanel } from "./components/SetlistPanel";
import { SongDetails } from "./components/SongDetails";
import { SongEditor } from "./components/SongEditor";
import { SongViewer } from "./components/SongViewer";
import { TransposeBar } from "./components/TransposeBar";
import { TypeScaleControls } from "./components/TypeScaleControls";
import { debounce } from "./lib/debounce";
import {
  addSetlistSong,
  beginEdit,
  cancelEdit,
  clearSong,
  createSetlist,
  createSong,
  deleteLibrarySong,
  deleteSetlist,
  duplicateLibrarySong,
  duplicateSetlist,
  getAppInfo,
  getCurrentSong,
  getEditorState,
  getSetlist,
  importBinary,
  importSong,
  importUrl,
  listLibrary,
  listSetlists,
  moveSetlistEntry,
  openLibrarySong,
  openSetlistEntry,
  openSetlistNeighbor,
  removeSetlistEntry,
  resetPerformanceKey,
  saveEdit,
  setPerformanceKey,
  toggleFavorite,
  transposeSong,
  updateMetadata,
  updateSetlistEntry,
  updateSetlistMeta,
} from "./lib/tauri";
import {
  applyTheme,
  applyTypeScale,
  loadTheme,
  loadTypeScale,
  persistTypeScale,
} from "./lib/theme";
import {
  isFullscreenHotkey,
  subscribeStageFullscreen,
  toggleStageFullscreen,
} from "./lib/stage";
import type {
  AppInfo,
  ImportFormat,
  LibraryList,
  LibrarySort,
  EditorSession,
  Setlist,
  SetlistSummary,
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
  const [bootNonce, setBootNonce] = useState(0);
  const [session, setSession] = useState<SongSession | null>(null);
  const [editor, setEditor] = useState<EditorSession | null>(null);
  const [library, setLibrary] = useState<LibraryList | null>(null);
  const [setlists, setSetlists] = useState<SetlistSummary[]>([]);
  const [openSetlist, setOpenSetlist] = useState<Setlist | null>(null);
  const [libraryTab, setLibraryTab] = useState<LibraryTab>("songs");
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [artistFilter, setArtistFilter] = useState("");
  const [keyFilter, setKeyFilter] = useState("");
  const [tagFilter, setTagFilter] = useState("");
  const [sort, setSort] = useState<LibrarySort>("title");
  const [importText, setImportText] = useState("");
  const [importFormat, setImportFormat] = useState<ImportFormat>("auto");
  const [importOpen, setImportOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [detailsDirty, setDetailsDirty] = useState(false);
  const [fullscreen, setFullscreen] = useState(false);
  const [live, setLive] = useState(false);
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
      search: debouncedSearch || null,
      artist: artistFilter || null,
      key: keyFilter || null,
      favoritesOnly,
      tag: tagFilter || null,
      sort,
    }),
    [debouncedSearch, artistFilter, keyFilter, favoritesOnly, tagFilter, sort],
  );

  const hasUnsavedWork = Boolean(editor?.dirty) || detailsDirty;

  const scheduleSearch = useMemo(
    () =>
      debounce((value: string) => {
        setDebouncedSearch(value);
      }, 200),
    [],
  );

  useEffect(() => {
    scheduleSearch(search);
    return () => scheduleSearch.cancel();
  }, [search, scheduleSearch]);

  async function refreshLibrary(): Promise<void> {
    setLibrary(await listLibrary(query));
  }

  async function refreshSetlists(): Promise<void> {
    setSetlists(await listSetlists());
  }

  async function refreshOpenSetlist(id: string): Promise<Setlist> {
    const next = await getSetlist(id);
    setOpenSetlist(next);
    return next;
  }

  useEffect(() => {
    let cancelled = false;
    setBoot({ status: "loading" });

    Promise.all([
      getAppInfo(),
      getCurrentSong(),
      listLibrary({}),
      getEditorState(),
      listSetlists(),
    ])
      .then(([info, current, songs, openEditor, listedSetlists]) => {
        if (cancelled) {
          return;
        }
        setBoot({ status: "ready", info });
        setLibrary(songs);
        setSetlists(listedSetlists);
        if (openEditor) {
          setEditor(openEditor);
          setImportOpen(false);
        }
        if (current) {
          setSession(current);
          if (!openEditor) {
            setImportOpen(false);
          }
          if (current.setlist) {
            void getSetlist(current.setlist.setlistId)
              .then((detail) => {
                if (!cancelled) {
                  setOpenSetlist(detail);
                  setLibraryTab("setlists");
                }
              })
              .catch(() => {
                /* setlist may have been deleted */
              });
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
  }, [bootNonce]);

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

  useEffect(() => subscribeStageFullscreen(setFullscreen), []);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (live) {
        return;
      }
      const target = event.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT" ||
          target.isContentEditable)
      ) {
        return;
      }
      if (!isFullscreenHotkey(event)) {
        return;
      }
      event.preventDefault();
      void toggleStageFullscreen().then(setFullscreen);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [live]);

  useEffect(() => {
    const onBeforeUnload = (event: BeforeUnloadEvent) => {
      if (!hasUnsavedWork) {
        return;
      }
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", onBeforeUnload);
    return () => window.removeEventListener("beforeunload", onBeforeUnload);
  }, [hasUnsavedWork]);

  const onDetailsDirtyChange = useCallback((dirty: boolean) => {
    setDetailsDirty(dirty);
  }, []);

  async function confirmLeaveDetails(): Promise<boolean> {
    if (!detailsDirty) {
      return true;
    }
    if (!window.confirm("Discard unsaved detail changes?")) {
      return false;
    }
    setDetailsDirty(false);
    return true;
  }

  async function confirmLeaveEditor(): Promise<boolean> {
    if (!(await confirmLeaveDetails())) {
      return false;
    }
    if (!editor) {
      return true;
    }
    if (editor.dirty && !window.confirm("Discard unsaved editor changes?")) {
      return false;
    }
    const remaining = await cancelEdit();
    setEditor(null);
    setSession(remaining);
    return true;
  }

  async function enterLive(next?: SongSession): Promise<void> {
    if (!(await confirmLeaveEditor())) {
      return;
    }
    if (next) {
      setSession(next);
    } else if (!session) {
      return;
    }
    setImportOpen(false);
    setActionError(null);
    setLive(true);
  }

  async function runAction(action: () => Promise<SongSession>): Promise<void> {
    setBusy(true);
    setActionError(null);
    try {
      const next = await action();
      setSession(next);
      await refreshLibrary();
      await refreshSetlists();
      const setlistId = next.setlist?.setlistId ?? openSetlist?.id;
      if (setlistId) {
        await refreshOpenSetlist(setlistId);
      }
    } catch (error: unknown) {
      setActionError(
        error instanceof Error ? error.message : "Something went wrong.",
      );
    } finally {
      setBusy(false);
    }
  }

  if (live && session) {
    return (
      <LiveMode
        session={session}
        keys={keys}
        busy={busy}
        error={actionError}
        restoreTheme={theme}
        restoreScale={typeScale}
        onExit={() => {
          setLive(false);
          setActionError(null);
        }}
        onPrev={() => void runAction(() => openSetlistNeighbor(-1))}
        onNext={() => void runAction(() => openSetlistNeighbor(1))}
        onTranspose={(semitones) =>
          void runAction(() => transposeSong(semitones))
        }
        onSelectKey={(key) => void runAction(() => setPerformanceKey(key))}
        onResetKey={() => void runAction(() => resetPerformanceKey())}
      />
    );
  }

  return (
    <div className="app-shell" aria-busy={busy}>
      <a className="skip-link" href="#main-content">
        Skip to content
      </a>
      <header className="app-header">
        <div className="brand">
          <p className="eyebrow">Songbook</p>
          <h1>Tonic</h1>
        </div>
        <div className="header-tools">
          <button
            type="button"
            className={fullscreen ? "chip chip--active" : "chip"}
            aria-pressed={fullscreen}
            aria-label={fullscreen ? "Exit fullscreen" : "Enter fullscreen"}
            title={
              fullscreen
                ? "Exit fullscreen (F11 or Alt+Enter)"
                : "Fullscreen (F11 or Alt+Enter)"
            }
            onClick={() => void toggleStageFullscreen().then(setFullscreen)}
          >
            {fullscreen ? "Exit full" : "Fullscreen"}
          </button>
          <div className="theme-toggle" role="group" aria-label="Theme">
            {(["dark", "light", "system"] as const).map((option) => (
              <button
                key={option}
                type="button"
                className={theme === option ? "chip chip--active" : "chip"}
                aria-pressed={theme === option}
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
            setlists={setlists}
            tab={libraryTab}
            activeId={editor?.songId ?? session?.song.id ?? null}
            activeSetlistId={openSetlist?.id ?? null}
            search={search}
            favoritesOnly={favoritesOnly}
            artist={artistFilter}
            songKey={keyFilter}
            tag={tagFilter}
            sort={sort}
            disabled={busy}
            onTabChange={setLibraryTab}
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
            onImport={() => {
              setImportOpen(true);
              setLibraryTab("songs");
            }}
            onNewSetlist={() =>
              void (async () => {
                if (!(await confirmLeaveEditor())) {
                  return;
                }
                setBusy(true);
                setActionError(null);
                try {
                  const next = await createSetlist();
                  setOpenSetlist(next);
                  setLibraryTab("setlists");
                  setImportOpen(false);
                  await refreshSetlists();
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
            onOpenSetlist={(id) =>
              void (async () => {
                if (!(await confirmLeaveEditor())) {
                  return;
                }
                setBusy(true);
                setActionError(null);
                try {
                  await refreshOpenSetlist(id);
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

        <main id="main-content" tabIndex={-1}>
          <div className="sr-only" aria-live="polite">
            {busy ? "Working…" : ""}
          </div>
          {boot.status === "loading" && (
            <p role="status">Connecting to the local engine…</p>
          )}

          {boot.status === "error" && (
            <div className="panel" role="alert">
              <h2>Engine unavailable</h2>
              <p>{boot.message}</p>
              <p className="hint">
                Run the desktop app with <code>npm run tauri dev</code> so the
                UI can talk to the Rust engine.
              </p>
              <button
                type="button"
                className="primary-button"
                onClick={() => setBootNonce((value) => value + 1)}
              >
                Retry connection
              </button>
            </div>
          )}

          {boot.status === "ready" && (
            <>
              <div className="toolbar">
                <button
                  type="button"
                  className={importOpen ? "text-button text-button--active" : "text-button"}
                  aria-expanded={importOpen}
                  aria-controls="import-panel"
                  onClick={() => setImportOpen((open) => !open)}
                >
                  {importOpen ? "Hide import" : "Import"}
                </button>
                {session && !editor && (
                  <button
                    type="button"
                    className="text-button"
                    onClick={() => void enterLive()}
                  >
                    Live
                  </button>
                )}
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
                      setDetailsDirty(false);
                      setImportOpen(false);
                      setActionError(null);
                    }}
                  >
                    Close song
                  </button>
                )}
                {session && !editor && (
                  <TransposeBar
                    displayKey={session.song.displayKey}
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
                  onImportBinary={(bytes, fileName) =>
                    void (async () => {
                      if (!(await confirmLeaveEditor())) {
                        return;
                      }
                      await runAction(async () => {
                        const next = await importBinary(bytes, fileName);
                        setEditor(null);
                        setImportOpen(false);
                        return next;
                      });
                    })()
                  }
                  onImportUrl={(url) =>
                    void (async () => {
                      if (!(await confirmLeaveEditor())) {
                        return;
                      }
                      await runAction(async () => {
                        const next = await importUrl(url);
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
                  <SongViewer
                    session={session}
                    disabled={busy}
                    onCapoChange={
                      session.setlist
                        ? (fret) =>
                            void (async () => {
                              if (!session.setlist) {
                                return;
                              }
                              setBusy(true);
                              setActionError(null);
                              try {
                                await updateSetlistEntry(
                                  session.setlist.setlistId,
                                  session.setlist.entryId,
                                  session.song.performanceKey,
                                  fret,
                                  session.setlist.entryNotes,
                                );
                                const current = await getCurrentSong();
                                if (current) {
                                  setSession(current);
                                }
                                await refreshOpenSetlist(
                                  session.setlist.setlistId,
                                );
                                await refreshSetlists();
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
                        : undefined
                    }
                  />
                  <SongDetails
                    session={session}
                    disabled={busy}
                    onDirtyChange={onDetailsDirtyChange}
                    onSave={(values) =>
                      void runAction(async () => {
                        const next = await updateMetadata({
                          title: values.title,
                          artist: values.artist || null,
                          album: values.album || null,
                          notes: values.notes || null,
                          tags: values.tags
                            .split(",")
                            .map((tag) => tag.trim())
                            .filter(Boolean),
                        });
                        setDetailsDirty(false);
                        return next;
                      })
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
                !importOpen &&
                !openSetlist && (
                  <div className="empty-state">
                    <p className="hint empty-hint">
                      Open a song from the library, or import a chart to get
                      started.
                    </p>
                    <button
                      type="button"
                      className="primary-button"
                      onClick={() => setImportOpen(true)}
                    >
                      Import chart
                    </button>
                  </div>
                )
              )}

              {boot.status === "ready" && openSetlist && !editor && (
                <SetlistPanel
                  key={openSetlist.id}
                  setlist={openSetlist}
                  songs={library?.songs ?? []}
                  keys={keys}
                  activeEntryId={session?.setlist?.entryId ?? null}
                  disabled={busy}
                  onRename={(name, notes, eventDate) =>
                    void (async () => {
                      setBusy(true);
                      setActionError(null);
                      try {
                        const next = await updateSetlistMeta(openSetlist.id, {
                          name,
                          notes: notes || null,
                          eventDate: eventDate || null,
                        });
                        setOpenSetlist(next);
                        await refreshSetlists();
                        if (session?.setlist?.setlistId === next.id) {
                          const current = await getCurrentSong();
                          if (current) {
                            setSession(current);
                          }
                        }
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
                  onAddSong={(songId) =>
                    void (async () => {
                      setBusy(true);
                      setActionError(null);
                      try {
                        setOpenSetlist(
                          await addSetlistSong(openSetlist.id, songId),
                        );
                        await refreshSetlists();
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
                  onRemoveEntry={(entryId) =>
                    void (async () => {
                      setBusy(true);
                      setActionError(null);
                      try {
                        setOpenSetlist(
                          await removeSetlistEntry(openSetlist.id, entryId),
                        );
                        await refreshSetlists();
                        const current = await getCurrentSong();
                        setSession(current);
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
                  onMoveEntry={(from, to) =>
                    void (async () => {
                      setBusy(true);
                      setActionError(null);
                      try {
                        setOpenSetlist(
                          await moveSetlistEntry(openSetlist.id, from, to),
                        );
                        if (session?.setlist?.setlistId === openSetlist.id) {
                          const current = await getCurrentSong();
                          if (current) {
                            setSession(current);
                          }
                        }
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
                  onOpenEntry={(entryId) =>
                    void (async () => {
                      if (!(await confirmLeaveEditor())) {
                        return;
                      }
                      await runAction(async () => {
                        const next = await openSetlistEntry(
                          openSetlist.id,
                          entryId,
                        );
                        setEditor(null);
                        setImportOpen(false);
                        return next;
                      });
                    })()
                  }
                  onUpdateEntry={(entryId, performanceKey, capoFret, notes) =>
                    void (async () => {
                      setBusy(true);
                      setActionError(null);
                      try {
                        setOpenSetlist(
                          await updateSetlistEntry(
                            openSetlist.id,
                            entryId,
                            performanceKey,
                            capoFret,
                            notes,
                          ),
                        );
                        await refreshSetlists();
                        if (session?.setlist?.entryId === entryId) {
                          const current = await getCurrentSong();
                          if (current) {
                            setSession(current);
                          }
                        }
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
                  onDuplicate={() =>
                    void (async () => {
                      setBusy(true);
                      setActionError(null);
                      try {
                        const copy = await duplicateSetlist(openSetlist.id);
                        setOpenSetlist(copy);
                        setLibraryTab("setlists");
                        await refreshSetlists();
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
                  onDelete={() => {
                    if (
                      !window.confirm(
                        `Delete setlist “${openSetlist.name}”? Songs stay in the library.`,
                      )
                    ) {
                      return;
                    }
                    void (async () => {
                      setBusy(true);
                      setActionError(null);
                      try {
                        await deleteSetlist(openSetlist.id);
                        setOpenSetlist(null);
                        await refreshSetlists();
                        const current = await getCurrentSong();
                        setSession(current);
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
                  onPerform={() =>
                    void (async () => {
                      if (
                        session?.setlist?.setlistId === openSetlist.id &&
                        session
                      ) {
                        await enterLive(session);
                        return;
                      }
                      const first = openSetlist.entries.find(
                        (entry) => !entry.missing,
                      );
                      if (!first) {
                        return;
                      }
                      if (!(await confirmLeaveEditor())) {
                        return;
                      }
                      setBusy(true);
                      setActionError(null);
                      try {
                        const next = await openSetlistEntry(
                          openSetlist.id,
                          first.id,
                        );
                        setEditor(null);
                        await refreshLibrary();
                        await refreshSetlists();
                        await enterLive(next);
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
            </>
          )}
        </main>
      </div>

      <footer>
        <p>
          Songs and setlists are saved locally on this device and stay available
          offline.
        </p>
      </footer>
    </div>
  );
}

export default App;
