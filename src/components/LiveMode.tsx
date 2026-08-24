import { useCallback, useEffect, useRef, useState } from "react";
import { CapoBadge } from "./CapoBadge";
import {
  IconChevronLeft,
  IconChevronRight,
  IconExit,
  IconEye,
  IconEyeOff,
  IconFullscreen,
  IconLock,
  IconPause,
  IconPlay,
  IconToTop,
  IconUnlock,
  IconWindowed,
} from "./icons";
import { SongViewer } from "./SongViewer";
import { TransposeModeToggle } from "./TransposeBar";
import {
  advanceScroll,
  clampScrollSpeed,
  DEFAULT_LIVE_SCALE,
  loadHideMeta,
  loadLiveScale,
  loadScrollSpeed,
  persistHideMeta,
  persistLiveScale,
  persistScrollSpeed,
  pinchScaleFromDistance,
  touchPairDistance,
} from "../lib/livePrefs";
import {
  isFullscreenHotkey,
  refreshKeepAwake,
  setKeepAwake,
  setStageFullscreen,
  subscribeStageFullscreen,
  toggleStageFullscreen,
} from "../lib/stage";
import { applyTheme, applyTypeScale, clampScale } from "../lib/theme";
import type {
  SongSession,
  ThemePreference,
  TransposeMode,
  TypeScale,
} from "../lib/types";

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
  onModeChange: (mode: TransposeMode) => void;
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
  onModeChange,
}: Props) {
  const scrollerRef = useRef<HTMLDivElement>(null);
  const swipeStart = useRef<{ x: number; y: number } | null>(null);
  const pinchRef = useRef<{
    startDist: number;
    base: TypeScale;
  } | null>(null);
  const pinchActive = useRef(false);
  const liveScaleRef = useRef<TypeScale>(DEFAULT_LIVE_SCALE);
  const speedRef = useRef(28);
  const [chromeVisible, setChromeVisible] = useState(true);
  const [chromePinned, setChromePinned] = useState(false);
  const [controlsLocked, setControlsLocked] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState(() =>
    typeof localStorage === "undefined" ? 28 : loadScrollSpeed(),
  );
  const [hideMeta, setHideMeta] = useState(() =>
    typeof localStorage === "undefined" ? false : loadHideMeta(),
  );
  const [fullscreen, setFullscreen] = useState(true);
  const [liveScale, setLiveScale] = useState<TypeScale>(() =>
    typeof localStorage === "undefined" ? DEFAULT_LIVE_SCALE : loadLiveScale(),
  );

  const setlist = session.setlist;
  const canPrev = Boolean(setlist && setlist.index > 0);
  const canNext = Boolean(setlist && setlist.index + 1 < setlist.total);
  const selectValue = session.song.displayKey ?? "";
  const chromeShown = chromeVisible || chromePinned;

  speedRef.current = speed;
  liveScaleRef.current = liveScale;

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
    applyTypeScale(liveScale);
    void setStageFullscreen(true);
    setFullscreen(true);
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

  useEffect(() => subscribeStageFullscreen(setFullscreen), []);

  useEffect(() => {
    applyTypeScale(liveScale);
    persistLiveScale(liveScale);
    const sheetZoom = liveScale.lyric / DEFAULT_LIVE_SCALE.lyric;
    document.documentElement.style.setProperty(
      "--sheet-zoom",
      String(Math.min(2.2, Math.max(0.55, sheetZoom))),
    );
  }, [liveScale]);

  useEffect(() => {
    return () => {
      document.documentElement.style.removeProperty("--sheet-zoom");
    };
  }, []);

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
    if (!chromeShown || controlsLocked || chromePinned) {
      return;
    }
    const timer = window.setTimeout(
      () => setChromeVisible(false),
      CHROME_IDLE_MS,
    );
    return () => window.clearTimeout(timer);
  }, [
    chromeShown,
    chromePinned,
    controlsLocked,
    playing,
    speed,
    hideMeta,
    session.song.id,
  ]);

  const pinChrome = useCallback(() => {
    setChromePinned(true);
    setChromeVisible(true);
  }, []);

  const releaseChromePin = useCallback((container: HTMLElement | null) => {
    requestAnimationFrame(() => {
      if (!container?.contains(document.activeElement)) {
        setChromePinned(false);
      }
    });
  }, []);

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

  const bumpScale = useCallback((delta: number) => {
    setLiveScale((scale) => ({
      lyric: clampScale(scale.lyric + delta),
      chord: clampScale(scale.chord + delta),
      section: clampScale(scale.section + delta),
    }));
  }, []);

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
      if (isFullscreenHotkey(event)) {
        event.preventDefault();
        void toggleStageFullscreen().then(setFullscreen);
        return;
      }
      const zoomMod = event.ctrlKey || event.metaKey;
      if (
        zoomMod &&
        (event.key === "-" ||
          event.key === "_" ||
          event.key === "+" ||
          event.key === "=")
      ) {
        event.preventDefault();
        showChrome();
        bumpScale(event.key === "-" || event.key === "_" ? -0.1 : 0.1);
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

    function onWheel(event: WheelEvent) {
      if (!(event.ctrlKey || event.metaKey)) {
        return;
      }
      event.preventDefault();
      showChrome();
      const step = event.deltaY === 0 ? 0 : event.deltaY > 0 ? -0.1 : 0.1;
      if (step !== 0) {
        bumpScale(step);
      }
    }

    window.addEventListener("keydown", onKey);
    window.addEventListener("wheel", onWheel, { passive: false });
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("wheel", onWheel);
    };
  }, [
    bumpScale,
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

  useEffect(() => {
    const el = scrollerRef.current;
    if (!el) {
      return;
    }

    const onTouchStart = (event: TouchEvent) => {
      if (event.touches.length >= 2) {
        const a = event.touches[0];
        const b = event.touches[1];
        if (!a || !b) {
          return;
        }
        pinchActive.current = true;
        swipeStart.current = null;
        pinchRef.current = {
          startDist: touchPairDistance(a, b),
          base: liveScaleRef.current,
        };
        showChrome();
        return;
      }
      const touch = event.touches[0];
      if (!touch || pinchActive.current) {
        return;
      }
      swipeStart.current = { x: touch.clientX, y: touch.clientY };
    };

    const onTouchMove = (event: TouchEvent) => {
      const pinch = pinchRef.current;
      if (!pinch || event.touches.length < 2) {
        return;
      }
      const a = event.touches[0];
      const b = event.touches[1];
      if (!a || !b) {
        return;
      }
      event.preventDefault();
      setLiveScale(
        pinchScaleFromDistance(
          pinch.base,
          pinch.startDist,
          touchPairDistance(a, b),
        ),
      );
      showChrome();
    };

    const onTouchEnd = (event: TouchEvent) => {
      if (event.touches.length >= 2) {
        return;
      }
      if (event.touches.length < 2) {
        pinchRef.current = null;
      }
      if (pinchActive.current) {
        if (event.touches.length === 0) {
          pinchActive.current = false;
        }
        swipeStart.current = null;
        return;
      }
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
    };

    el.addEventListener("touchstart", onTouchStart, { passive: true });
    el.addEventListener("touchmove", onTouchMove, { passive: false });
    el.addEventListener("touchend", onTouchEnd);
    el.addEventListener("touchcancel", onTouchEnd);
    return () => {
      el.removeEventListener("touchstart", onTouchStart);
      el.removeEventListener("touchmove", onTouchMove);
      el.removeEventListener("touchend", onTouchEnd);
      el.removeEventListener("touchcancel", onTouchEnd);
    };
  }, [busy, canNext, canPrev, onNext, onPrev, showChrome]);

  return (
    <div
      className={
        controlsLocked
          ? "live-shell live-shell--locked"
          : chromeShown
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
        onPointerDown={showChrome}
      >
        <div key={session.song.id} className="song-stage song-stage--live">
          <SongViewer session={session} hideMeta={hideMeta} live />
        </div>
      </div>

      {controlsLocked && (
        <button
          type="button"
          className="live-lock icon-button"
          aria-pressed={true}
          aria-label="Unlock controls"
          title="Unlock controls (L)"
          onClick={toggleLock}
        >
          <IconUnlock />
        </button>
      )}

      <div
        className="live-chrome"
        aria-hidden={controlsLocked || !chromeShown}
        onFocusCapture={pinChrome}
        onBlurCapture={(event) => releaseChromePin(event.currentTarget)}
        {...(controlsLocked || !chromeShown
          ? ({ inert: true } as { inert: boolean })
          : {})}
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
              {session.transposeMode === "capo" && session.capoFret != null ? (
                <>
                  {" · "}
                  <CapoBadge
                    fret={session.capoFret}
                    playedKey={session.playedKey}
                  />
                </>
              ) : setlist?.capoFret != null ? (
                <>
                  {" · "}
                  <CapoBadge
                    fret={setlist.capoFret}
                    playedKey={setlist.playedKey}
                  />
                </>
              ) : null}
            </p>
          </div>
          <button
            type="button"
            className="icon-button"
            aria-pressed={fullscreen}
            aria-label={fullscreen ? "Exit fullscreen" : "Enter fullscreen"}
            title={
              fullscreen
                ? "Exit fullscreen (F11 or Alt+Enter)"
                : "Fullscreen (F11 or Alt+Enter)"
            }
            onClick={() => void toggleStageFullscreen().then(setFullscreen)}
          >
            {fullscreen ? <IconWindowed /> : <IconFullscreen />}
          </button>
          <button
            type="button"
            className="icon-button"
            aria-pressed={false}
            aria-label="Lock controls"
            title="Lock controls (L)"
            onClick={toggleLock}
          >
            <IconLock />
          </button>
          <button
            type="button"
            className="icon-button"
            aria-label="Exit live"
            title="Exit live (Esc)"
            onClick={onExit}
          >
            <IconExit />
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
            className="icon-button"
            aria-label="Previous"
            title="Previous song"
            disabled={!canPrev || busy}
            onClick={onPrev}
          >
            <IconChevronLeft />
          </button>
          <button
            type="button"
            className="icon-button"
            aria-pressed={playing}
            aria-label={playing ? "Pause scroll" : "Auto-scroll"}
            title={playing ? "Pause auto-scroll" : "Start auto-scroll"}
            onClick={() => setPlaying((value) => !value)}
          >
            {playing ? <IconPause /> : <IconPlay />}
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
          <button
            type="button"
            className="icon-button"
            aria-label="Jump to top"
            title="Jump to top"
            onClick={scrollToTop}
          >
            <IconToTop />
          </button>
          <TransposeModeToggle
            mode={session.transposeMode}
            disabled={busy}
            onChange={onModeChange}
          />
          <button
            type="button"
            className="icon-button"
            aria-label={
              session.transposeMode === "capo"
                ? "Move capo down a fret"
                : "Transpose down a semitone"
            }
            disabled={
              busy ||
              (session.transposeMode === "capo" && (session.capoFret ?? 0) <= 0)
            }
            onClick={() => onTranspose(-1)}
          >
            −
          </button>
          <label className="key-select-label">
            <span className="sr-only">Performance key</span>
            <select
              value={selectValue}
              disabled={busy || keys.length === 0}
              onMouseDown={pinChrome}
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
            aria-label={
              session.transposeMode === "capo"
                ? "Move capo up a fret"
                : "Transpose up a semitone"
            }
            disabled={
              busy ||
              (session.transposeMode === "capo" && (session.capoFret ?? 0) >= 12)
            }
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
            className="icon-button"
            aria-pressed={hideMeta}
            aria-label={hideMeta ? "Show info" : "Hide info"}
            title={hideMeta ? "Show song info" : "Hide song info"}
            onClick={() => setHideMeta((value) => !value)}
          >
            {hideMeta ? <IconEye /> : <IconEyeOff />}
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
            className="icon-button"
            aria-label="Next"
            title="Next song"
            disabled={!canNext || busy}
            onClick={onNext}
          >
            <IconChevronRight />
          </button>
        </div>
      </div>
    </div>
  );
}
