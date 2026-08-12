export type AppInfo = {
  name: string;
  version: string;
  phase: number;
  domainEngine: string;
  domainVersion: string;
  persistenceHealthy: boolean;
  performanceKeys: string[];
};

export type ImportFormat = "auto" | "chordPro" | "plainText";

export type ChordStatus =
  "fullyRecognized" | "partiallyRecognized" | "unrecognized";

export type ChordView = {
  symbol: string;
  written: string;
  lyricIndex: number;
  column: number | null;
  status: ChordStatus | string;
};

export type LineView = {
  lyrics: string;
  chords: ChordView[];
  annotations: string[];
};

export type SectionView = {
  label: string;
  lines: LineView[];
};

export type SongView = {
  id: string;
  title: string;
  artist: string | null;
  album: string | null;
  originalKey: string | null;
  performanceKey: string | null;
  tempoBpm: number | null;
  timeSignature: string | null;
  notes: string | null;
  sourceFormat: string;
  sections: SectionView[];
};

export type WarningView = {
  kind: string;
  message: string;
  line: number | null;
};

export type SongSession = {
  song: SongView;
  warnings: WarningView[];
  summaryMessage: string | null;
  semitoneOffset: number;
};

export type ThemePreference = "dark" | "light" | "system";

export type TypeScale = {
  lyric: number;
  chord: number;
  section: number;
};
