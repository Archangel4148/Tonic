import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import type { SongSession } from "./lib/types";

const mockedInvoke = vi.mocked(invoke);

const appInfo = {
  name: "Tonic",
  version: "0.1.0",
  phase: 6,
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
    const handler = handlers[String(cmd)];
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
    });

    render(<App />);

    expect(
      await screen.findByRole("heading", { name: "Tonic" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText(/chart text/i)).toBeInTheDocument();
    expect(screen.getByRole("searchbox")).toBeInTheDocument();
    fireEvent.click(screen.getByText("Engine"));
    expect(screen.getByText(/tonic-domain v0\.1\.0/i)).toBeInTheDocument();
    expect(screen.getByText(/local library healthy/i)).toBeInTheDocument();
  });

  it("imports a pasted chart and shows aligned chords", async () => {
    mockIpc({
      app_info: appInfo,
      current_song: null,
      library_list: emptyLibrary,
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
      screen.getByLabelText("Amazing grace, how sweet the sound"),
    ).toBeInTheDocument();
    const verse = screen.getByLabelText("Verse 1");
    expect(verse).toHaveTextContent("G");
    expect(verse).toHaveTextContent("D");
  });

  it("transposes through the engine without rewriting lyrics", async () => {
    mockIpc({
      app_info: appInfo,
      current_song: demoSession,
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
      screen.getByLabelText("Amazing grace, how sweet the sound"),
    ).toBeInTheDocument();
    expect(mockedInvoke).toHaveBeenCalledWith("transpose_song", {
      semitones: 1,
    });
  });

  it("opens a library song from the sidebar", async () => {
    mockIpc({
      app_info: appInfo,
      current_song: null,
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

  it("surfaces an error when the engine is unavailable", async () => {
    mockedInvoke.mockRejectedValue(new Error("IPC unavailable"));

    render(<App />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "IPC unavailable",
    );
  });
});
