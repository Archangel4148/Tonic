import { useEffect, useRef, useState } from "react";
import {
  clampScrollSpeed,
  DEFAULT_LIVE_SCALE,
  DEFAULT_SCROLL_SPEED,
  loadHideMeta,
  loadLiveScale,
  loadScrollSpeed,
  MAX_SCROLL_SPEED,
  MIN_SCROLL_SPEED,
  persistHideMeta,
  persistLiveScale,
  persistScrollSpeed,
} from "../lib/livePrefs";
import { DEFAULT_TYPE_SCALE } from "../lib/theme";
import type {
  AppInfo,
  LibraryInfo,
  ThemePreference,
  TypeScale,
} from "../lib/types";
import { TypeScaleControls } from "./TypeScaleControls";

type Props = {
  open: boolean;
  onClose: () => void;
  appInfo: AppInfo;
  libraryInfo: LibraryInfo | null;
  theme: ThemePreference;
  onThemeChange: (theme: ThemePreference) => void;
  typeScale: TypeScale;
  onTypeScaleChange: (scale: TypeScale) => void;
  onClearLibrary: () => void;
  busy?: boolean;
};

export function SettingsPanel({
  open,
  onClose,
  appInfo,
  libraryInfo,
  theme,
  onThemeChange,
  typeScale,
  onTypeScaleChange,
  onClearLibrary,
  busy = false,
}: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [liveScale, setLiveScale] = useState<TypeScale>(() => loadLiveScale());
  const [scrollSpeed, setScrollSpeed] = useState(() => loadScrollSpeed());
  const [hideMeta, setHideMeta] = useState(() => loadHideMeta());

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) {
      return;
    }
    if (open && !dialog.open) {
      dialog.showModal();
      setLiveScale(loadLiveScale());
      setScrollSpeed(loadScrollSpeed());
      setHideMeta(loadHideMeta());
    } else if (!open && dialog.open) {
      dialog.close();
    }
  }, [open]);

  return (
    <dialog
      ref={dialogRef}
      className="settings-dialog"
      aria-labelledby="settings-title"
      onClose={onClose}
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
    >
      <form method="dialog" className="settings-dialog__inner panel">
        <header className="settings-dialog__header">
          <div>
            <p className="eyebrow">Preferences</p>
            <h2 id="settings-title">Settings</h2>
          </div>
          <button
            type="submit"
            className="icon-button"
            aria-label="Close settings"
          >
            ×
          </button>
        </header>

        <section className="settings-section">
          <h3 className="settings-section__title">Appearance</h3>
          <label className="field-label" htmlFor="settings-theme">
            Theme
          </label>
          <select
            id="settings-theme"
            className="settings-select"
            value={theme}
            onChange={(event) =>
              onThemeChange(event.target.value as ThemePreference)
            }
          >
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="system">System</option>
          </select>
        </section>

        <section className="settings-section">
          <h3 className="settings-section__title">Text size</h3>
          <p className="settings-hint">
            Default sizes for the song viewer and editor.
          </p>
          <TypeScaleControls scale={typeScale} onChange={onTypeScaleChange} />
          <button
            type="button"
            className="text-button"
            onClick={() => onTypeScaleChange(DEFAULT_TYPE_SCALE)}
          >
            Reset text size
          </button>
        </section>

        <section className="settings-section">
          <h3 className="settings-section__title">Live mode defaults</h3>
          <p className="settings-hint">
            Starting values when you enter live mode. You can still adjust them
            during a performance.
          </p>
          <label className="field-label" htmlFor="settings-live-speed">
            Auto-scroll speed ({scrollSpeed})
          </label>
          <input
            id="settings-live-speed"
            type="range"
            min={MIN_SCROLL_SPEED}
            max={MAX_SCROLL_SPEED}
            value={scrollSpeed}
            onChange={(event) => {
              const next = clampScrollSpeed(Number(event.target.value));
              setScrollSpeed(next);
              persistScrollSpeed(next);
            }}
          />
          <label className="settings-checkbox">
            <input
              type="checkbox"
              checked={hideMeta}
              onChange={(event) => {
                setHideMeta(event.target.checked);
                persistHideMeta(event.target.checked);
              }}
            />
            Hide song info by default in live mode
          </label>
          <p className="field-label">Live text size</p>
          <TypeScaleControls
            scale={liveScale}
            onChange={(scale) => {
              setLiveScale(scale);
              persistLiveScale(scale);
            }}
          />
          <button
            type="button"
            className="text-button"
            onClick={() => {
              persistScrollSpeed(DEFAULT_SCROLL_SPEED);
              persistHideMeta(false);
              persistLiveScale(DEFAULT_LIVE_SCALE);
              setScrollSpeed(DEFAULT_SCROLL_SPEED);
              setHideMeta(false);
              setLiveScale(DEFAULT_LIVE_SCALE);
            }}
          >
            Reset live defaults
          </button>
        </section>

        <section className="settings-section">
          <h3 className="settings-section__title">Storage</h3>
          {libraryInfo ? (
            <dl className="settings-stats">
              <div>
                <dt>Songs</dt>
                <dd>{libraryInfo.songCount}</dd>
              </div>
              <div>
                <dt>Setlists</dt>
                <dd>{libraryInfo.setlistCount}</dd>
              </div>
              <div>
                <dt>Status</dt>
                <dd>
                  {libraryInfo.persistenceHealthy
                    ? "Library healthy"
                    : "Unavailable"}
                </dd>
              </div>
              {libraryInfo.libraryPath && (
                <div className="settings-stats__path">
                  <dt>Location</dt>
                  <dd>{libraryInfo.libraryPath}</dd>
                </div>
              )}
            </dl>
          ) : (
            <p className="settings-hint">Loading library info…</p>
          )}
          <button
            type="button"
            className="text-button text-button--danger"
            disabled={busy || !libraryInfo}
            onClick={() => {
              const songCount = libraryInfo?.songCount ?? 0;
              const setlistCount = libraryInfo?.setlistCount ?? 0;
              const total = songCount + setlistCount;
              if (total === 0) {
                window.alert("Your library is already empty.");
                return;
              }
              const confirmed = window.confirm(
                `Clear local library?\n\nThis will permanently delete ${songCount} song(s) and ${setlistCount} setlist(s).\n\nThis cannot be undone.`,
              );
              if (confirmed) {
                onClearLibrary();
              }
            }}
          >
            Clear local library
          </button>
        </section>

        <section className="settings-section">
          <h3 className="settings-section__title">About</h3>
          <div className="settings-about-brand">
            <img
              className="settings-about-mark"
              src="/tonic-icon.png"
              alt=""
              width={48}
              height={48}
              decoding="async"
            />
            <div>
              <p className="settings-about-name">Tonic</p>
              <p className="settings-hint">A musician&apos;s local songbook</p>
            </div>
          </div>
          <dl className="settings-stats">
            <div>
              <dt>Application</dt>
              <dd>
                {appInfo.name} v{appInfo.version}
              </dd>
            </div>
            <div>
              <dt>Engine</dt>
              <dd>
                {appInfo.domainEngine} v{appInfo.domainVersion}
              </dd>
            </div>
            <div>
              <dt>Phase</dt>
              <dd>{appInfo.phase}</dd>
            </div>
          </dl>
        </section>
      </form>
    </dialog>
  );
}
