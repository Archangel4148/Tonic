export type AppInfo = {
  name: string;
  version: string;
  phase: number;
  domainEngine: string;
  domainVersion: string;
  persistenceHealthy: boolean;
  performanceKeys: string[];
};

export type ImportFormat = "auto" | "chordPro" | "plainText" | "musicXml";

export type ChordStatus =
  "fullyRecognized" | "partiallyRecognized" | "unrecognized";

export type TransposeMode = "chords" | "capo";

export type ChordView = {
  symbol: string;
  written: string;
  sounding: string;
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
  displayKey: string | null;
  tempoBpm: number | null;
  timeSignature: string | null;
  notes: string | null;
  sourceFormat: string;
  hasScore: boolean;
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
  favorite: boolean;
  tags: string[];
  setlist: SetlistContext | null;
  transposeMode: TransposeMode;
  capoFret: number | null;
  playedKey: string | null;
  sheetMusicXml: string | null;
};

export type SetlistContext = {
  setlistId: string;
  setlistName: string;
  entryId: string;
  index: number;
  total: number;
  capoFret: number | null;
  entryNotes: string | null;
  playedKey: string | null;
};

export type SetlistSummary = {
  id: string;
  name: string;
  notes: string | null;
  eventDate: string | null;
  songCount: number;
  updatedAt: number | null;
};

export type SetlistEntry = {
  id: string;
  songId: string;
  title: string;
  artist: string | null;
  missing: boolean;
  songKey: string | null;
  performanceKey: string | null;
  capoFret: number | null;
  notes: string | null;
};

export type Setlist = {
  id: string;
  name: string;
  notes: string | null;
  eventDate: string | null;
  entries: SetlistEntry[];
};

export type SetlistMetaUpdate = {
  name: string;
  notes: string | null;
  eventDate: string | null;
};

export type LibrarySort =
  "title" | "artist" | "recentOpened" | "recentModified";

export type LibraryQuery = {
  search?: string | null;
  artist?: string | null;
  key?: string | null;
  favoritesOnly?: boolean | null;
  tag?: string | null;
  sort?: LibrarySort | string | null;
};

export type LibrarySongSummary = {
  id: string;
  title: string;
  artist: string | null;
  album: string | null;
  originalKey: string | null;
  performanceKey: string | null;
  favorite: boolean;
  tags: string[];
  lastOpenedAt: number | null;
  lastModifiedAt: number | null;
};

export type LibraryList = {
  songs: LibrarySongSummary[];
  recents: LibrarySongSummary[];
  artists: string[];
  keys: string[];
  tags: string[];
};

export type MetadataUpdate = {
  title: string;
  artist: string | null;
  album: string | null;
  notes: string | null;
  tags: string[];
};

export type EditorSession = {
  songId: string;
  dirty: boolean;
  isNew: boolean;
  title: string;
  artist: string | null;
  album: string | null;
  originalKey: string | null;
  tempoBpm: number | null;
  timeSignature: string | null;
  notes: string | null;
  tags: string[];
  warnings: WarningView[];
  summaryMessage: string | null;
  chartText: string;
};

export type EditorMetaUpdate = {
  title: string;
  artist: string | null;
  album: string | null;
  originalKey: string | null;
  tempoBpm: number | null;
  timeSignature: string | null;
  notes: string | null;
  tags: string[];
};

export type EditorSaveResult = {
  session: SongSession;
  editor: EditorSession;
};

export type LibraryInfo = {
  libraryPath: string | null;
  songCount: number;
  setlistCount: number;
  persistenceHealthy: boolean;
};

export type OpenLibraryFolderResult = {
  path: string;
  opened: boolean;
  message: string;
};

export type LibraryStorageStatus = {
  libraryPath: string | null;
  kind: string;
  documentsPath: string | null;
  documentsWritable: boolean;
  hasAllFilesAccess: boolean;
  canUseDocuments: boolean;
  hint: string;
};

export type LibraryReloadResult = {
  session: SongSession | null;
  editor: EditorSession | null;
};

export type ThemePreference =
  | "dark"
  | "light"
  | "system"
  | "ink"
  | "forest"
  | "ocean"
  | "slate"
  | "wine"
  | "amethyst"
  | "frost"
  | "moss"
  | "stone";

export type TypeScale = {
  lyric: number;
  chord: number;
  section: number;
};
