import { describe, expect, it, vi } from "vitest";
import { debounce } from "./debounce";

describe("debounce", () => {
  it("delays calls and keeps only the latest arguments", () => {
    vi.useFakeTimers();
    const fn = vi.fn();
    const delayed = debounce(fn, 100);
    delayed("a");
    delayed("b");
    expect(fn).not.toHaveBeenCalled();
    vi.advanceTimersByTime(100);
    expect(fn).toHaveBeenCalledTimes(1);
    expect(fn).toHaveBeenCalledWith("b");
    vi.useRealTimers();
  });

  it("cancel prevents a pending call", () => {
    vi.useFakeTimers();
    const fn = vi.fn();
    const delayed = debounce(fn, 100);
    delayed("a");
    delayed.cancel();
    vi.advanceTimersByTime(100);
    expect(fn).not.toHaveBeenCalled();
    vi.useRealTimers();
  });
});
