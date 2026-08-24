import type { ThemePreference, TypeScale } from "./types";

const THEME_KEY = "tonic-theme";
const SCALE_KEY = "tonic-type-scale";

export const THEME_IDS = [
  "system",
  "dark",
  "light",
  "ink",
  "forest",
  "ocean",
  "slate",
  "wine",
  "amethyst",
  "frost",
  "moss",
  "stone",
] as const satisfies readonly ThemePreference[];

export type ThemeOption = {
  id: ThemePreference;
  label: string;
  group: "auto" | "dark" | "light";
  swatch: { bg: string; accent: string };
};

export const THEME_OPTIONS: ThemeOption[] = [
  {
    id: "system",
    label: "System",
    group: "auto",
    swatch: { bg: "#12100e", accent: "#f6f1e8" },
  },
  {
    id: "dark",
    label: "Ember",
    group: "dark",
    swatch: { bg: "#12100e", accent: "#d4a373" },
  },
  {
    id: "ink",
    label: "Ink",
    group: "dark",
    swatch: { bg: "#0c0c0d", accent: "#e8e4dc" },
  },
  {
    id: "forest",
    label: "Pine",
    group: "dark",
    swatch: { bg: "#101610", accent: "#8fbf90" },
  },
  {
    id: "ocean",
    label: "Harbor",
    group: "dark",
    swatch: { bg: "#0c141c", accent: "#7eb6d0" },
  },
  {
    id: "slate",
    label: "Slate",
    group: "dark",
    swatch: { bg: "#181a1f", accent: "#9aabc4" },
  },
  {
    id: "wine",
    label: "Wine",
    group: "dark",
    swatch: { bg: "#160e12", accent: "#d4a0a8" },
  },
  {
    id: "amethyst",
    label: "Amethyst",
    group: "dark",
    swatch: { bg: "#141018", accent: "#b89ad0" },
  },
  {
    id: "light",
    label: "Parchment",
    group: "light",
    swatch: { bg: "#f6f1e8", accent: "#8a5a2b" },
  },
  {
    id: "frost",
    label: "Frost",
    group: "light",
    swatch: { bg: "#eef2f6", accent: "#2d5a78" },
  },
  {
    id: "moss",
    label: "Moss",
    group: "light",
    swatch: { bg: "#e8eee6", accent: "#3d5c3a" },
  },
  {
    id: "stone",
    label: "Stone",
    group: "light",
    swatch: { bg: "#e9eae8", accent: "#2f5f5b" },
  },
];

export const DEFAULT_TYPE_SCALE: TypeScale = {
  lyric: 1.2,
  chord: 1.05,
  section: 0.82,
};

export function isThemePreference(value: string): value is ThemePreference {
  return (THEME_IDS as readonly string[]).includes(value);
}

export function loadTheme(): ThemePreference {
  const stored = localStorage.getItem(THEME_KEY);
  if (stored && isThemePreference(stored)) {
    return stored;
  }
  return "dark";
}

export function applyTheme(theme: ThemePreference, persist = true): void {
  document.documentElement.setAttribute("data-theme", theme);
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
