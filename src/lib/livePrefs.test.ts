import { describe, expect, it } from "vitest";
import { advanceScroll } from "./livePrefs";

describe("advanceScroll", () => {
  it("moves by fractional pixels so slow speeds are not rounded away", () => {
    const slow = advanceScroll(0, 1000, 28, 0.016);
    expect(slow.finished).toBe(false);
    expect(slow.position).toBeCloseTo(0.448, 3);

    const faster = advanceScroll(0, 1000, 90, 0.016);
    expect(faster.position).toBeCloseTo(1.44, 3);
    expect(faster.position).toBeGreaterThan(slow.position);
  });

  it("stops at the bottom without jumping past max", () => {
    const done = advanceScroll(998, 1000, 90, 0.05);
    expect(done.position).toBe(1000);
    expect(done.finished).toBe(true);
  });
});
