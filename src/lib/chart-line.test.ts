import { describe, expect, it } from "vitest";
import { splitChartLine } from "./chart-line";
import type { ChordView } from "./types";

function chord(
  symbol: string,
  lyricIndex: number,
  column: number | null = null,
): ChordView {
  return {
    symbol,
    sounding: symbol,
    lyricIndex,
    column,
    status: "fullyRecognized",
  };
}

describe("splitChartLine", () => {
  it("places chords over matching lyric indices", () => {
    const lyrics = "Amazing grace, how sweet the sound";
    const how = "Amazing grace, how ".length;
    const segments = splitChartLine(lyrics, [chord("G", 0), chord("D", how)]);

    expect(segments[0]?.chords.map((item) => item.symbol)).toEqual(["G"]);
    expect(segments[0]?.text.startsWith("Amazing")).toBe(true);
    expect(segments.some((segment) => segment.chords[0]?.symbol === "D")).toBe(
      true,
    );
    expect(segments.map((segment) => segment.text).join("")).toBe(lyrics);
  });

  it("keeps lyric-only lines as a single segment", () => {
    const segments = splitChartLine("Standalone lyric", []);
    expect(segments).toEqual([{ text: "Standalone lyric", chords: [] }]);
  });

  it("renders chord-only lines", () => {
    const segments = splitChartLine("", [chord("C", 0), chord("G", 11)]);
    expect(segments).toHaveLength(1);
    expect(segments[0]?.chords.map((item) => item.symbol)).toEqual(["C", "G"]);
  });
});
