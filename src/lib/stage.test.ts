import { describe, expect, it } from "vitest";
import { isFullscreenHotkey } from "./stage";

describe("isFullscreenHotkey", () => {
  it("matches F11 and Alt+Enter", () => {
    expect(
      isFullscreenHotkey(
        new KeyboardEvent("keydown", { key: "F11" }),
      ),
    ).toBe(true);
    expect(
      isFullscreenHotkey(
        new KeyboardEvent("keydown", { key: "Enter", altKey: true }),
      ),
    ).toBe(true);
    expect(
      isFullscreenHotkey(new KeyboardEvent("keydown", { key: "Enter" })),
    ).toBe(false);
    expect(
      isFullscreenHotkey(new KeyboardEvent("keydown", { key: "f" })),
    ).toBe(false);
  });
});
