import { describe, expect, it } from "vitest";
import {
  advanceScroll,
  pinchScaleFromDistance,
  touchPairDistance,
} from "./livePrefs";

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

describe("pinch scale", () => {
  it("measures distance between touches", () => {
    expect(
      touchPairDistance({ clientX: 0, clientY: 0 }, { clientX: 3, clientY: 4 }),
    ).toBe(5);
  });

  it("scales all text channels from the pinch start snapshot", () => {
    const base = { lyric: 1.9, chord: 1.65, section: 1.05 };
    const bigger = pinchScaleFromDistance(base, 100, 200);
    expect(bigger.lyric).toBe(2.4); // clamped
    expect(bigger.chord).toBeGreaterThan(base.chord);
    expect(bigger.section).toBeGreaterThan(base.section);

    const smaller = pinchScaleFromDistance(base, 100, 50);
    expect(smaller.lyric).toBeLessThan(base.lyric);
    expect(smaller.chord).toBeLessThan(base.chord);
  });
});
