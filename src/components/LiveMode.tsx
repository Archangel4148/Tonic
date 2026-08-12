import { useCallback, useEffect, useRef, useState } from "react";
import { SongViewer } from "./SongViewer";
import {
  advanceScroll,
  clampScrollSpeed,
  loadHideMeta,
  loadLiveScale,
  loadScrollSpeed,
  persistHideMeta,
  persistLiveScale,
  persistScrollSpeed,
} from "../lib/livePrefs";
import {
  refreshKeepAwake,
  setKeepAwake,
  setStageFullscreen,
} from "../lib/stage";
import { applyTheme, applyTypeScale, clampScale } from "../lib/theme";
import type { SongSession, ThemePreference, TypeScale } from "../lib/types";

type Props = {
  session: SongSession;
  keys: string[];
  busy?: boolean;
  error?: string | null;
  restoreTheme: ThemePreference;
  restoreScale: TypeScale;
  onExit: () => void;
  onPrev: () => void;
  onNext: () => void;
  onTranspose: (semitones: number) => void;
  onSelectKey: (key: string) => void;
  onResetKey: () => void;
};

const CHROME_IDLE_MS = 2800;
const SWIPE_MIN = 56;

export function LiveMode({
  session,
  keys,
  busy,
  error,
  restoreTheme,
  restoreScale,
  onExit,
  onPrev,
  onNext,
  onTranspose,
  onSelectKey,
  onResetKey,
}: Props) {
  const scrollerRef = useRef<HTMLDivElement>(null);
  const swipeStart = useRef<{ x: number; y: number } | null>(null);
  const speedRef = useRef(28);
  const [chromeVisible, setChromeVisible] = useState(true);
  const [controlsLocked, setControlsLocked] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState(() =>
    typeof localStorage === "undefined" ? 28 : loadScrollSpeed(),
  );
  const [hideMeta, setHideMeta] = useState(() =>
    typeof localStorage === "undefined" ? false : loadHideMeta(),
  );
  const [liveScale, setLiveScale] = useState<TypeScale>(() =>
    typeof localStorage === "undefined"
      ? { lyric: 1.9, chord: 1.65, section: 1.05 }
      : loadLiveScale(),
  );

  const setlist = session.setlist;
  const canPrev = Boolean(setlist && setlist.index > 0);
  const canNext = Boolean(setlist && setlist.index + 1 < setlist.total);
  const selectValue =
    session.song.performanceKey ?? session.song.originalKey ?? "";

  speedRef.current = speed;

  const showChrome = useCallback(() => {
    if (!controlsLocked) {
      setChromeVisible(true);
    }
  }, [controlsLocked]);

  const toggleLock = useCallback(() => {
    setControlsLocked((locked) => {
      const next = !locked;
      if (!next) {
        setChromeVisible(true);
      }
      return next;
    });
  }, []);

  function scrollToTop(): void {
    const el = scrollerRef.current;
    if (el) {
      el.scrollTop = 0;
    }
  }

  useEffect(() => {
    document.documentElement.setAttribute("data-live", "true");
    applyTheme("dark", false);
    applyTypeScale(liveScale);
    void setStageFullscreen(true);
    void setKeepAwake(true);
    return () => {
      document.documentElement.removeAttribute("data-live");
      applyTheme(restoreTheme);
      applyTypeScale(restoreScale);
      void setStageFullscreen(false);
      void setKeepAwake(false);
    };
    // Enter/exit live once per mount; restore values are captured on enter.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    applyTypeScale(liveScale);
    persistLiveScale(liveScale);
  }, [liveScale]);

  useEffect(() => {
    persistScrollSpeed(speed);
  }, [speed]);

  useEffect(() => {
    persistHideMeta(hideMeta);
  }, [hideMeta]);

  useEffect(() => {
    const onVisibility = () => {
      if (document.visibilityState === "visible") {
        void refreshKeepAwake();
      }
    };
    document.addEventListener("visibilitychange", onVisibility);
    return () => document.removeEventListener("visibilitychange", onVisibility);
  }, []);

  useEffect(() => {
    scrollToTop();
    setPlaying(false);
  }, [session.song.id, session.setlist?.entryId]);

  useEffect(() => {
    if (!chromeVisible || controlsLocked) {
      return;
    }
    const timer = window.setTimeout(
      () => setChromeVisible(false),
      CHROME_IDLE_MS,
    );
    return () => window.clearTimeout(timer);
  }, [
    chromeVisible,
    controlsLocked,
    playing,
    speed,
    hideMeta,
    session.song.id,
  ]);

  useEffect(() => {
    if (!playing) {
      return;
    }
    const el = scrollerRef.current;
    if (!el) {
      return;
    }
    let last = performance.now();
    let position = el.scrollTop;
    let frame = 0;
    const tick = (now: number) => {
      const dt = Math.min(0.05, (now - last) / 1000);
      last = now;
      const max = el.scrollHeight - el.clientHeight;
      if (Math.abs(el.scrollTop - position) > 2) {
        position = el.scrollTop;
      }
      const next = advanceScroll(position, max, speedRef.current, dt);
      position = next.position;
      el.scrollTop = position;
      if (next.finished) {
        setPlaying(false);
        return;
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [playing]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      const target = event.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT")
      ) {
        return;
      }
      if (event.key === "l" || event.key === "L") {
        event.preventDefault();
        toggleLock();
        return;
      }
      showChrome();
      if (event.key === "Escape") {
        event.preventDefault();
        onExit();
        return;
      }
      if (event.key === " " || event.key === "Spacebar") {
        event.preventDefault();
        setPlaying((value) => !value);
        return;
      }
      if (event.key === "ArrowLeft" || event.key === "PageUp") {
        event.preventDefault();
        if (canPrev && !busy) {
          onPrev();
        }
        return;
      }
      if (event.key === "ArrowRight" || event.key === "PageDown") {
        event.preventDefault();
        if (canNext && !busy) {
          onNext();
        }
        return;
      }
      if (event.key === "Home") {
        event.preventDefault();
        scrollToTop();
        return;
      }
      if (event.key === "-" || event.key === "_") {
        event.preventDefault();
        onTranspose(-1);
        return;
      }
      if (event.key === "+" || event.key === "=") {
        event.preventDefault();
        onTranspose(1);
        return;
      }
      if (event.key === "[") {
        event.preventDefault();
        setSpeed((value) => clampScrollSpeed(value - 4));
        return;
      }
      if (event.key === "]") {
        event.preventDefault();
        setSpeed((value) => clampScrollSpeed(value + 4));
        return;
      }
      if (event.key === "m" || event.key === "M") {
        event.preventDefault();
        setHideMeta((value) => !value);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [
    busy,
    canNext,
    canPrev,
    controlsLocked,
    onExit,
    onNext,
    onPrev,
    onTranspose,
    showChrome,
    toggleLock,
  ]);

  function bumpScale(delta: number): void {
    setLiveScale((scale) => ({
      lyric: clampScale(scale.lyric + delta),
      chord: clampScale(scale.chord + delta),
      section: clampScale(scale.section + delta),
    }));
  }

  function onTouchStart(event: React.TouchEvent<HTMLDivElement>): void {
    const touch = event.changedTouches[0];
    if (!touch) {
      return;
    }
    swipeStart.current = { x: touch.clientX, y: touch.clientY };
  }

  function onTouchEnd(event: React.TouchEvent<HTMLDivElement>): void {
    const start = swipeStart.current;
    swipeStart.current = null;
    const touch = event.changedTouches[0];
    if (!start || !touch) {
      return;
    }
    const dx = touch.clientX - start.x;
    const dy = touch.clientY - start.y;
    if (Math.abs(dx) < SWIPE_MIN || Math.abs(dx) < Math.abs(dy) * 1.4) {
      showChrome();
      return;
    }
    if (dx < 0 && canNext && !busy) {
      onNext();
    } else if (dx > 0 && canPrev && !busy) {
      onPrev();
    }
  }

  return (
    <div
      className={
        controlsLocked
          ? "live-shell live-shell--locked"
          : chromeVisible
            ? "live-shell live-shell--chrome"
            : "live-shell"
      }
      onMouseMove={showChrome}
    >
      {setlist && (
        <div
          className="live-progress"
          role="progressbar"
          aria-label="Setlist progress"
          aria-valuemin={1}
          aria-valuemax={setlist.total}
          aria-valuenow={setlist.index + 1}
        >
          <span
            style={{
              width: `${((setlist.index + 1) / Math.max(1, setlist.total)) * 100}%`,
            }}
          />
        </div>
      )}

      <div
        ref={scrollerRef}
        className="live-scroller"
        onTouchStart={onTouchStart}
        onTouchEnd={onTouchEnd}
        onPointerDown={showChrome}
      >
        <SongViewer session={session} hideMeta={hideMeta} live />
      </div>

      {controlsLocked && (
        <button
          type="button"
          className="live-lock"
          aria-pressed={true}
          aria-label="Unlock controls"
          title="Unlock controls (L)"
          onClick={toggleLock}
        >
          Unlock
        </button>
      )}

      <div
        className="live-chrome"
        aria-hidden={controlsLocked || !chromeVisible}
      >
        <div className="live-chrome-bar live-chrome-bar--top">
          <div className="live-identity">
            <p className="live-title">{session.song.title}</p>
            <p className="live-meta">
              {setlist
                ? `${setlist.setlistName} · ${setlist.index + 1}/${setlist.total}`
                : "Single song"}
              {session.song.performanceKey
                ? ` · ${session.song.performanceKey}`
                : ""}
              {setlist?.capoFret != null ? ` · capo ${setlist.capoFret}` : ""}
              {setlist?.playedKey ? ` · played ${setlist.playedKey}` : ""}
            </p>
          </div>
          <button
            type="button"
            className="text-button"
            aria-pressed={false}
            aria-label="Lock controls"
            title="Lock controls (L)"
            onClick={toggleLock}
          >
            Lock
          </button>
          <button type="button" className="text-button" onClick={onExit}>
            Exit live
          </button>
        </div>

        {error && !controlsLocked && (
          <p className="live-error" role="alert">
            {error}
          </p>
        )}

        <div className="live-chrome-bar live-chrome-bar--bottom">
          <button
            type="button"
            className="text-button"
            disabled={!canPrev || busy}
            onClick={onPrev}
          >
            Previous
          </button>
          <button
            type="button"
            className="text-button"
            aria-pressed={playing}
            onClick={() => setPlaying((value) => !value)}
          >
            {playing ? "Pause scroll" : "Auto-scroll"}
          </button>
          <label className="live-speed">
            Speed {speed}
            <input
              type="range"
              min={8}
              max={90}
              value={speed}
              aria-label="Auto-scroll speed"
              onChange={(event) =>
                setSpeed(clampScrollSpeed(Number(event.target.value)))
              }
            />
          </label>
          <button type="button" className="text-button" onClick={scrollToTop}>
            Top
          </button>
          <button
            type="button"
            className="icon-button"
            aria-label="Transpose down a semitone"
            disabled={busy}
            onClick={() => onTranspose(-1)}
          >
            −
          </button>
          <label className="key-select-label">
            <span className="sr-only">Performance key</span>
            <select
              value={selectValue}
              disabled={busy || keys.length === 0}
              onChange={(event) => onSelectKey(event.target.value)}
            >
              {!selectValue && <option value="">Key</option>}
              {selectValue && !keys.includes(selectValue) && (
                <option value={selectValue}>{selectValue}</option>
              )}
              {keys.map((key) => (
                <option key={key} value={key}>
                  {key}
                </option>
              ))}
            </select>
          </label>
          <button
            type="button"
            className="icon-button"
            aria-label="Transpose up a semitone"
            disabled={busy}
            onClick={() => onTranspose(1)}
          >
            +
          </button>
          <button
            type="button"
            className="text-button"
            disabled={busy || session.semitoneOffset === 0}
            onClick={onResetKey}
          >
            Reset
          </button>
          <button
            type="button"
            className="text-button"
            aria-pressed={hideMeta}
            onClick={() => setHideMeta((value) => !value)}
          >
            {hideMeta ? "Show info" : "Hide info"}
          </button>
          <button
            type="button"
            className="icon-button icon-button--small"
            aria-label="Decrease live text size"
            onClick={() => bumpScale(-0.1)}
          >
            −
          </button>
          <button
            type="button"
            className="icon-button icon-button--small"
            aria-label="Increase live text size"
            onClick={() => bumpScale(0.1)}
          >
            +
          </button>
          <button
            type="button"
            className="text-button"
            disabled={!canNext || busy}
            onClick={onNext}
          >
            Next
          </button>
        </div>
      </div>
    </div>
  );
}
