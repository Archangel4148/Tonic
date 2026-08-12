import type { ChordView } from "./types";

export type ChartSegment = {
  text: string;
  chords: ChordView[];
};

/** Split lyrics at chord `lyricIndex` values (Unicode scalars). */
export function splitChartLine(
  lyrics: string,
  chords: ChordView[],
): ChartSegment[] {
  const chars = [...lyrics];

  if (chars.length === 0) {
    if (chords.length === 0) {
      return [];
    }
    return [{ text: "", chords: [...chords] }];
  }

  const at = new Map<number, ChordView[]>();
  for (const chord of chords) {
    const index = Math.max(0, Math.min(chord.lyricIndex, chars.length));
    const existing = at.get(index) ?? [];
    existing.push(chord);
    at.set(index, existing);
  }

  const cuts = [...new Set([0, ...at.keys(), chars.length])].sort(
    (a, b) => a - b,
  );

  const segments: ChartSegment[] = [];
  for (let i = 0; i < cuts.length - 1; i += 1) {
    const start = cuts[i];
    const end = cuts[i + 1];
    segments.push({
      text: chars.slice(start, end).join(""),
      chords: at.get(start) ?? [],
    });
  }

  if (at.has(chars.length)) {
    segments.push({ text: "", chords: at.get(chars.length) ?? [] });
  }

  return segments;
}
