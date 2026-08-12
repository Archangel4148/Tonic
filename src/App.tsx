import { useEffect, useState } from "react";
import { getAppInfo } from "./lib/tauri";
import type { AppInfo } from "./lib/types";
import "./App.css";

function phaseLabel(phase: number): string {
  switch (phase) {
    case 1:
      return " — Foundation";
    case 2:
      return " — Music engine";
    case 3:
      return " — Song model";
    default:
      return "";
  }
}

type LoadState =
  | { status: "loading" }
  | { status: "ready"; info: AppInfo }
  | { status: "error"; message: string };

function App() {
  const [state, setState] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    let cancelled = false;

    getAppInfo()
      .then((info) => {
        if (!cancelled) {
          setState({ status: "ready", info });
        }
      })
      .catch((error: unknown) => {
        const message =
          error instanceof Error
            ? error.message
            : "Could not reach the Tonic engine.";
        if (!cancelled) {
          setState({ status: "error", message });
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="app-shell">
      <header className="app-header">
        <p className="eyebrow">Songbook</p>
        <h1>Tonic</h1>
        <p className="tagline">A musician&apos;s digital songbook</p>
      </header>

      <main>
        {state.status === "loading" && (
          <p role="status">Connecting to the local engine…</p>
        )}

        {state.status === "error" && (
          <div className="panel" role="alert">
            <h2>Engine unavailable</h2>
            <p>{state.message}</p>
            <p className="hint">
              Run the desktop app with <code>npm run tauri dev</code> so the UI
              can talk to the Rust engine.
            </p>
          </div>
        )}

        {state.status === "ready" && (
          <section className="panel" aria-labelledby="engine-status-heading">
            <h2 id="engine-status-heading">Engine status</h2>
            <dl className="status-list">
              <div>
                <dt>Application</dt>
                <dd>
                  {state.info.name} v{state.info.version}
                </dd>
              </div>
              <div>
                <dt>Phase</dt>
                <dd>
                  {state.info.phase}
                  {phaseLabel(state.info.phase)}
                </dd>
              </div>
              <div>
                <dt>Domain engine</dt>
                <dd>
                  {state.info.domainEngine} v{state.info.domainVersion}
                </dd>
              </div>
              <div>
                <dt>Persistence</dt>
                <dd>
                  {state.info.persistenceHealthy
                    ? "In-memory stub healthy"
                    : "Unavailable"}
                </dd>
              </div>
            </dl>
          </section>
        )}
      </main>

      <footer>
        <p>Library, editor, and live mode arrive in later phases.</p>
      </footer>
    </div>
  );
}

export default App;
