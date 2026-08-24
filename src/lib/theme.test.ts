import { afterEach, describe, expect, it } from "vitest";
import { applyTheme, isThemePreference, loadTheme, THEME_OPTIONS } from "./theme";

describe("theme palettes", () => {
  afterEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("keeps ember, parchment, and system plus extra palettes", () => {
    const ids = THEME_OPTIONS.map((option) => option.id);
    expect(ids).toContain("dark");
    expect(ids).toContain("light");
    expect(ids).toContain("system");
    expect(ids.length).toBeGreaterThan(6);
  });

  it("defaults to ember dark and ignores unknown stored values", () => {
    expect(loadTheme()).toBe("dark");
    localStorage.setItem("tonic-theme", "neon");
    expect(loadTheme()).toBe("dark");
    expect(isThemePreference("forest")).toBe(true);
    expect(isThemePreference("neon")).toBe(false);
  });

  it("applies and persists a named palette", () => {
    applyTheme("ocean");
    expect(document.documentElement.getAttribute("data-theme")).toBe("ocean");
    expect(localStorage.getItem("tonic-theme")).toBe("ocean");
    expect(loadTheme()).toBe("ocean");
  });
});
