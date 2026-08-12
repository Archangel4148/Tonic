import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import type { EditorSession, SongSession } from "./lib/types";

const mockedInvoke = vi.mocked(invoke);

const appInfo = {
  name: "Tonic",
  version: "0.1.0",
  phase: 11,
  domainEngine: "tonic-domain",
  domainVersion: "0.1.0",
  persistenceHealthy: true,
  performanceKeys: ["C", "G", "A", "Am"],
};

const emptyLibrary = {
  songs: [],
  recents: [],
  artists: [],
  keys: [],
  tags: [],
};

const demoSession: SongSession = {
  song: {
    id: "session-1",
    title: "Amazing Grace",
    artist: "Traditional",
    album: null,
    originalKey: "G",
    performanceKey: "G",
    tempoBpm: 72,
    timeSignature: "3/4",
    notes: null,
    sourceFormat: "chordPro",
    hasScore: false,
    sections: [
      {
        label: "Verse 1",
        lines: [
          {
            lyrics: "Amazing grace, how sweet the sound",
            chords: [
              {
                symbol: "G",
                written: "G",
                lyricIndex: 0,
                column: null,
                status: "fullyRecognized",
              },
              {
                symbol: "D",
                written: "D",
                lyricIndex: "Amazing grace, how ".length,
                column: null,
                status: "fullyRecognized",
              },
            ],
            annotations: [],
          },
        ],
      },
    ],
  },
  warnings: [],
  summaryMessage: null,
  semitoneOffset: 0,
  favorite: false,
  tags: ["hymn"],
  setlist: null,
  sheetMusicXml: null,
};

const transposedSession: SongSession = {
  ...demoSession,
  semitoneOffset: 2,
  song: {
    ...demoSession.song,
    performanceKey: "A",
    sections: [
      {
        label: "Verse 1",
        lines: [
          {
            lyrics: "Amazing grace, how sweet the sound",
            chords: [
              {
                symbol: "A",
                written: "G",
                lyricIndex: 0,
                column: null,
                status: "fullyRecognized",
              },
              {
                symbol: "E",
                written: "D",
                lyricIndex: "Amazing grace, how ".length,
                column: null,
                status: "fullyRecognized",
              },
            ],
            annotations: [],
          },
        ],
      },
    ],
  },
};

function mockIpc(
  handlers: Record<string, unknown | ((args?: unknown) => unknown)>,
) {
  mockedInvoke.mockImplementation(async (cmd, args) => {
    const handler = { setlist_list: [], ...handlers }[String(cmd)];
    if (typeof handler === "function") {
      return handler(args);
    }
    if (handler !== undefined) {
      return handler;
    }
    throw new Error(`unexpected command ${String(cmd)}`);
  });
}

describe("App", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    localStorage.clear();
  });

  it("renders import UI after the engine responds", async () => {
    mockIpc({
      app_info: appInfo,
      current_song: null,
      library_list: emptyLibrary,
      editor_state: null,
    });

    render(<App />);

    expect(
      await screen.findByRole("heading", { name: "Tonic" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText(/chart text/i)).toBeInTheDocument();
    expect(
      screen.getByRole("option", { name: "MusicXML" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("searchbox")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /enter fullscreen/i }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByText("Engine"));
    expect(screen.getByText(/tonic-domain v0\.1\.0/i)).toBeInTheDocument();
    expect(screen.getByText(/local library healthy/i)).toBeInTheDocument();
  });

  it("imports a pasted chart and shows aligned chords", async () => {
    mockIpc({
      app_info: appInfo,
      current_song: null,
      library_list: emptyLibrary,
      editor_state: null,
      import_song: demoSession,
    });

    render(<App />);
    const textarea = await screen.findByLabelText(/chart text/i);
    fireEvent.change(textarea, {
      value: "{title: Amazing Grace}\n[G]Amazing grace",
      target: { value: "{title: Amazing Grace}\n[G]Amazing grace" },
    });
    fireEvent.click(screen.getByRole("button", { name: /import song/i }));

    expect(
      await screen.findByRole("heading", { name: "Amazing Grace" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Traditional")).toBeInTheDocument();
    expect(screen.getByText("Verse 1")).toBeInTheDocument();
    expect(
      screen.getByLabelText(/Amazing grace, how sweet the sound\. Chords:/i),
    ).toBeInTheDocument();
    const verse = screen.getByLabelText("Verse 1");
    expect(verse).toHaveTextContent("G");
    expect(verse).toHaveTextContent("D");
  });

  it("transposes through the engine without rewriting lyrics", async () => {
    mockIpc({
      app_info: appInfo,
      current_song: demoSession,
      editor_state: null,
      library_list: {
        ...emptyLibrary,
        songs: [
          {
            id: demoSession.song.id,
            title: demoSession.song.title,
            artist: demoSession.song.artist,
            album: demoSession.song.album,
            originalKey: demoSession.song.originalKey,
            performanceKey: demoSession.song.performanceKey,
            favorite: false,
            tags: ["hymn"],
            lastOpenedAt: 1,
            lastModifiedAt: 1,
          },
        ],
        recents: [],
        artists: ["Traditional"],
        keys: ["G"],
        tags: ["hymn"],
      },
      transpose_song: transposedSession,
    });

    render(<App />);
    expect(
      await screen.findByRole("heading", { name: "Amazing Grace" }),
    ).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", { name: /transpose up a semitone/i }),
    );

    await waitFor(() => {
      expect(document.body).toHaveTextContent(/Now\s+A/);
    });
    const verse = screen.getByLabelText("Verse 1");
    expect(verse).toHaveTextContent("A");
    expect(verse).toHaveTextContent("E");
    expect(
      screen.getByLabelText(/Amazing grace, how sweet the sound\. Chords:/i),
    ).toBeInTheDocument();
    expect(mockedInvoke).toHaveBeenCalledWith("transpose_song", {
      semitones: 1,
    });
  });

  it("opens a library song from the sidebar", async () => {
    mockIpc({
      app_info: appInfo,
      current_song: null,
      editor_state: null,
      library_list: {
        ...emptyLibrary,
        songs: [
          {
            id: demoSession.song.id,
            title: demoSession.song.title,
            artist: demoSession.song.artist,
            album: demoSession.song.album,
            originalKey: demoSession.song.originalKey,
            performanceKey: demoSession.song.performanceKey,
            favorite: false,
            tags: ["hymn"],
            lastOpenedAt: 1,
            lastModifiedAt: 1,
          },
        ],
        artists: ["Traditional"],
        keys: ["G"],
        tags: ["hymn"],
      },
      library_open: demoSession,
    });

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: /Amazing Grace Traditional/i }),
    );
    expect(
      await screen.findByRole("heading", { name: "Amazing Grace" }),
    ).toBeInTheDocument();
    expect(mockedInvoke).toHaveBeenCalledWith("library_open", {
      id: "session-1",
    });
  });

  it("opens the editor for a new song", async () => {
    const newEditor: EditorSession = {
      songId: "song-1",
      dirty: true,
      isNew: true,
      title: "Untitled",
      artist: null,
      album: null,
      originalKey: null,
      tempoBpm: null,
      timeSignature: null,
      notes: null,
      tags: [],
      warnings: [],
      summaryMessage: null,
      sections: [
        {
          label: "Verse",
          kind: "verse",
          number: null,
          customName: null,
          lines: [{ lyrics: "", chords: [], annotation: null }],
        },
      ],
    };
    mockIpc({
      app_info: appInfo,
      current_song: null,
      editor_state: null,
      library_list: emptyLibrary,
      editor_create: newEditor,
    });

    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "New song" }));
    expect(
      await screen.findByRole("heading", { name: "New song" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Tag chord at caret" }),
    ).toBeInTheDocument();
    expect(mockedInvoke).toHaveBeenCalledWith("editor_create");
  });

  it("opens a setlist entry without copying the song id", async () => {
    const setlistSession: SongSession = {
      ...demoSession,
      song: { ...demoSession.song, performanceKey: "Bb" },
      setlist: {
        setlistId: "setlist-1",
        setlistName: "Friday gig",
        entryId: "entry-1",
        index: 0,
        total: 2,
        capoFret: 2,
        entryNotes: "slow intro",
        playedKey: "Ab",
      },
    };
    mockIpc({
      app_info: appInfo,
      current_song: null,
      editor_state: null,
      library_list: {
        ...emptyLibrary,
        songs: [
          {
            id: demoSession.song.id,
            title: demoSession.song.title,
            artist: demoSession.song.artist,
            album: demoSession.song.album,
            originalKey: demoSession.song.originalKey,
            performanceKey: demoSession.song.performanceKey,
            favorite: false,
            tags: ["hymn"],
            lastOpenedAt: 1,
            lastModifiedAt: 1,
          },
        ],
        artists: ["Traditional"],
        keys: ["G"],
        tags: ["hymn"],
      },
      setlist_list: [
        {
          id: "setlist-1",
          name: "Friday gig",
          notes: null,
          eventDate: null,
          songCount: 2,
          updatedAt: 1,
        },
      ],
      setlist_get: {
        id: "setlist-1",
        name: "Friday gig",
        notes: null,
        eventDate: null,
        entries: [
          {
            id: "entry-1",
            songId: demoSession.song.id,
            title: demoSession.song.title,
            artist: demoSession.song.artist,
            missing: false,
            songKey: "G",
            performanceKey: "Bb",
            capoFret: 2,
            notes: "slow intro",
          },
          {
            id: "entry-2",
            songId: demoSession.song.id,
            title: demoSession.song.title,
            artist: demoSession.song.artist,
            missing: false,
            songKey: "G",
            performanceKey: null,
            capoFret: null,
            notes: null,
          },
        ],
      },
      setlist_open_entry: setlistSession,
    });

    render(<App />);
    fireEvent.click(await screen.findByRole("tab", { name: "Setlists" }));
    fireEvent.click(await screen.findByRole("button", { name: /Friday gig/i }));
    fireEvent.click(
      await screen.findByRole("button", { name: /1\. Amazing Grace/i }),
    );
    expect(await screen.findByLabelText("Setlist context")).toHaveTextContent(
      /Friday gig/,
    );
    expect(screen.getByText(/Played/)).toHaveTextContent("Ab");
    expect(mockedInvoke).toHaveBeenCalledWith("setlist_open_entry", {
      setlistId: "setlist-1",
      entryId: "entry-1",
    });
  });

  it("enters live mode and returns with escape", async () => {
    mockIpc({
      app_info: appInfo,
      current_song: demoSession,
      editor_state: null,
      library_list: emptyLibrary,
    });

    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Live" }));
    expect(
      await screen.findByRole("button", { name: "Exit live" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Auto-scroll" }),
    ).toBeInTheDocument();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(
      await screen.findByRole("button", { name: "Live" }),
    ).toBeInTheDocument();
  });

  it("locks live controls while hotkeys still work", async () => {
    mockIpc({
      app_info: appInfo,
      current_song: demoSession,
      editor_state: null,
      library_list: emptyLibrary,
    });

    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Live" }));
    fireEvent.click(
      await screen.findByRole("button", { name: "Lock controls" }),
    );
    expect(
      screen.queryByRole("button", { name: "Exit live" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Unlock controls" }),
    ).toBeInTheDocument();
    fireEvent.keyDown(window, { key: "l" });
    expect(
      await screen.findByRole("button", { name: "Exit live" }),
    ).toBeInTheDocument();
  });

  it("advances the setlist from live mode", async () => {
    const setlistSession: SongSession = {
      ...demoSession,
      setlist: {
        setlistId: "setlist-1",
        setlistName: "Friday gig",
        entryId: "entry-1",
        index: 0,
        total: 2,
        capoFret: null,
        entryNotes: null,
        playedKey: null,
      },
    };
    const nextSession: SongSession = {
      ...demoSession,
      song: { ...demoSession.song, title: "Second Song" },
      setlist: {
        ...setlistSession.setlist!,
        entryId: "entry-2",
        index: 1,
      },
    };
    mockIpc({
      app_info: appInfo,
      current_song: setlistSession,
      editor_state: null,
      library_list: emptyLibrary,
      setlist_get: {
        id: "setlist-1",
        name: "Friday gig",
        notes: null,
        eventDate: null,
        entries: [],
      },
      setlist_open_neighbor: nextSession,
    });

    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Live" }));
    fireEvent.click(await screen.findByRole("button", { name: "Next" }));
    expect(
      await screen.findByRole("heading", { name: "Second Song" }),
    ).toBeInTheDocument();
    expect(mockedInvoke).toHaveBeenCalledWith("setlist_open_neighbor", {
      delta: 1,
    });
  });

  it("surfaces an error when the engine is unavailable", async () => {
    mockedInvoke.mockRejectedValue(new Error("IPC unavailable"));

    render(<App />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "IPC unavailable",
    );
    expect(
      screen.getByRole("button", { name: /retry connection/i }),
    ).toBeInTheDocument();
  });

  it("exposes a skip link and empty-state guidance", async () => {
    mockIpc({
      app_info: appInfo,
      current_song: null,
      library_list: emptyLibrary,
      editor_state: null,
    });

    render(<App />);
    expect(
      await screen.findByRole("link", { name: /skip to content/i }),
    ).toHaveAttribute("href", "#main-content");
    fireEvent.click(screen.getByRole("button", { name: /hide import/i }));
    expect(
      screen.getByText(/Open a song, import a chart, or create a setlist/i),
    ).toBeInTheDocument();
  });

  it("warns before unload when the editor is dirty", async () => {
    const dirtyEditor: EditorSession = {
      songId: "song-1",
      dirty: true,
      isNew: true,
      title: "Untitled",
      artist: null,
      album: null,
      originalKey: null,
      tempoBpm: null,
      timeSignature: null,
      notes: null,
      tags: [],
      warnings: [],
      summaryMessage: null,
      sections: [
        {
          label: "Verse",
          kind: "verse",
          number: null,
          customName: null,
          lines: [{ lyrics: "", chords: [], annotation: null }],
        },
      ],
    };
    mockIpc({
      app_info: appInfo,
      current_song: null,
      library_list: emptyLibrary,
      editor_state: dirtyEditor,
    });

    render(<App />);
    expect(
      await screen.findByRole("heading", { name: "New song" }),
    ).toBeInTheDocument();

    const event = new Event("beforeunload", { cancelable: true });
    Object.defineProperty(event, "returnValue", {
      writable: true,
      value: undefined,
    });
    window.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
  });
});
