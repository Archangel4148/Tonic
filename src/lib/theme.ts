import type { ThemePreference, TypeScale } from "./types";

const THEME_KEY = "tonic-theme";
const SCALE_KEY = "tonic-type-scale";

export const DEFAULT_TYPE_SCALE: TypeScale = {
  lyric: 1.2,
  chord: 1.05,
  section: 0.82,
};

export function loadTheme(): ThemePreference {
  const stored = localStorage.getItem(THEME_KEY);
  if (stored === "dark" || stored === "light" || stored === "system") {
    return stored;
  }
  return "dark";
}

export function applyTheme(theme: ThemePreference, persist = true): void {
  const root = document.documentElement;
  if (theme === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", theme);
  }
  if (persist) {
    localStorage.setItem(THEME_KEY, theme);
  }
}

export function loadTypeScale(): TypeScale {
  try {
    const raw = localStorage.getItem(SCALE_KEY);
    if (!raw) {
      return DEFAULT_TYPE_SCALE;
    }
    const parsed = JSON.parse(raw) as Partial<TypeScale>;
    return {
      lyric: clampScale(parsed.lyric ?? DEFAULT_TYPE_SCALE.lyric),
      chord: clampScale(parsed.chord ?? DEFAULT_TYPE_SCALE.chord),
      section: clampScale(parsed.section ?? DEFAULT_TYPE_SCALE.section),
    };
  } catch {
    return DEFAULT_TYPE_SCALE;
  }
}

export function persistTypeScale(scale: TypeScale): void {
  localStorage.setItem(SCALE_KEY, JSON.stringify(scale));
}

export function applyTypeScale(scale: TypeScale): void {
  const root = document.documentElement;
  root.style.setProperty("--lyric-size", `${scale.lyric}rem`);
  root.style.setProperty("--chord-size", `${scale.chord}rem`);
  root.style.setProperty("--section-size", `${scale.section}rem`);
}

export function clampScale(value: number): number {
  return Math.min(2.4, Math.max(0.7, Math.round(value * 20) / 20));
}
