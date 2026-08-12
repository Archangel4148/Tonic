import { useEffect, useMemo, useState } from "react";
import { EngineStatus } from "./components/EngineStatus";
import { ImportPanel } from "./components/ImportPanel";
import { SongViewer } from "./components/SongViewer";
import { TransposeBar } from "./components/TransposeBar";
import { TypeScaleControls } from "./components/TypeScaleControls";
import {
  clearSong,
  getAppInfo,
  getCurrentSong,
  importSong,
  resetPerformanceKey,
  setPerformanceKey,
  transposeSong,
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

  useEffect(() => {
    let cancelled = false;

    Promise.all([getAppInfo(), getCurrentSong()])
      .then(([info, current]) => {
        if (cancelled) {
          return;
        }
        setBoot({ status: "ready", info });
        if (current) {
          setSession(current);
          setImportOpen(false);
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
    applyTheme(theme);
  }, [theme]);

  useEffect(() => {
    applyTypeScale(typeScale);
    persistTypeScale(typeScale);
  }, [typeScale]);

  async function runAction(action: () => Promise<SongSession>): Promise<void> {
    setBusy(true);
    setActionError(null);
    try {
      const next = await action();
      setSession(next);
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
              {session && (
                <button
                  type="button"
                  className="text-button"
                  onClick={async () => {
                    await clearSong();
                    setSession(null);
                    setImportOpen(true);
                    setActionError(null);
                  }}
                >
                  Clear song
                </button>
              )}
              {session && (
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

            {importOpen && (
              <ImportPanel
                text={importText}
                format={importFormat}
                busy={busy}
                onTextChange={setImportText}
                onFormatChange={setImportFormat}
                onImport={(text, format) =>
                  void runAction(async () => {
                    setImportText(text);
                    setImportFormat(format);
                    const next = await importSong(text, format);
                    setImportOpen(false);
                    return next;
                  })
                }
              />
            )}

            {session ? (
              <SongViewer session={session} />
            ) : (
              !importOpen && (
                <p className="hint empty-hint">
                  Import a chart to read chords and lyrics.
                </p>
              )
            )}
          </>
        )}
      </main>

      <footer>
        <p>
          {session
            ? "Library save arrives in a later phase. This song lives in memory until you quit."
            : "Paste a chart, then change key without losing alignment."}
        </p>
      </footer>
    </div>
  );
}

export default App;
