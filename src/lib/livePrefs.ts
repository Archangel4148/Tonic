import type { TypeScale } from "./types";
import { clampScale } from "./theme";

const SPEED_KEY = "tonic-live-scroll-speed";
const HIDE_META_KEY = "tonic-live-hide-meta";
const SCALE_KEY = "tonic-live-type-scale";

export const DEFAULT_LIVE_SCALE: TypeScale = {
  lyric: 1.9,
  chord: 1.65,
  section: 1.05,
};

export const MIN_SCROLL_SPEED = 8;
export const MAX_SCROLL_SPEED = 90;
export const DEFAULT_SCROLL_SPEED = 28;

export function loadScrollSpeed(): number {
  const raw = Number(localStorage.getItem(SPEED_KEY));
  if (!Number.isFinite(raw)) {
    return DEFAULT_SCROLL_SPEED;
  }
  return clampScrollSpeed(raw);
}

export function persistScrollSpeed(speed: number): void {
  localStorage.setItem(SPEED_KEY, String(clampScrollSpeed(speed)));
}

export function clampScrollSpeed(value: number): number {
  return Math.min(
    MAX_SCROLL_SPEED,
    Math.max(MIN_SCROLL_SPEED, Math.round(value)),
  );
}

/** Fractional scroll step so speeds below 1px/frame still move. */
export function advanceScroll(
  position: number,
  max: number,
  speedPxPerSec: number,
  dtSeconds: number,
): { position: number; finished: boolean } {
  if (max <= 0 || position >= max) {
    return { position: Math.max(0, max), finished: true };
  }
  const next = Math.min(max, position + speedPxPerSec * Math.max(0, dtSeconds));
  return { position: next, finished: next >= max };
}

export function loadHideMeta(): boolean {
  return localStorage.getItem(HIDE_META_KEY) === "1";
}

export function persistHideMeta(hide: boolean): void {
  localStorage.setItem(HIDE_META_KEY, hide ? "1" : "0");
}

export function loadLiveScale(): TypeScale {
  try {
    const raw = localStorage.getItem(SCALE_KEY);
    if (!raw) {
      return DEFAULT_LIVE_SCALE;
    }
    const parsed = JSON.parse(raw) as Partial<TypeScale>;
    return {
      lyric: clampScale(parsed.lyric ?? DEFAULT_LIVE_SCALE.lyric),
      chord: clampScale(parsed.chord ?? DEFAULT_LIVE_SCALE.chord),
      section: clampScale(parsed.section ?? DEFAULT_LIVE_SCALE.section),
    };
  } catch {
    return DEFAULT_LIVE_SCALE;
  }
}

export function persistLiveScale(scale: TypeScale): void {
  localStorage.setItem(SCALE_KEY, JSON.stringify(scale));
}

/** Distance between two touch points (for pinch gestures). */
export function touchPairDistance(
  a: { clientX: number; clientY: number },
  b: { clientX: number; clientY: number },
): number {
  return Math.hypot(a.clientX - b.clientX, a.clientY - b.clientY);
}

/** Scale lyric/chord/section together from a pinch start snapshot. */
export function pinchScaleFromDistance(
  base: TypeScale,
  startDistance: number,
  currentDistance: number,
): TypeScale {
  if (!(startDistance > 0) || !(currentDistance > 0)) {
    return base;
  }
  const factor = currentDistance / startDistance;
  return {
    lyric: clampScale(base.lyric * factor),
    chord: clampScale(base.chord * factor),
    section: clampScale(base.section * factor),
  };
}
