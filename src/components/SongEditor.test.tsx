import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SongEditor } from "./SongEditor";
import type { EditorSession } from "../lib/types";

const editor: EditorSession = {
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
      lines: [
        {
          lyrics: "Hello world",
          chords: [
            { symbol: "C", lyricIndex: 0, status: "fullyRecognized" },
            { symbol: "Xyz", lyricIndex: 6, status: "unrecognized" },
          ],
          annotation: null,
        },
      ],
    },
  ],
};

describe("SongEditor", () => {
  it("shows lyrics, chord tags, and save/cancel", () => {
    render(
      <SongEditor
        editor={editor}
        keys={["C", "G"]}
        onChange={vi.fn()}
        onSave={vi.fn(async () => undefined)}
        onCancel={vi.fn()}
      />,
    );

    expect(screen.getByRole("heading", { name: "New song" })).toBeInTheDocument();
    expect(screen.getByDisplayValue("Hello world")).toBeInTheDocument();
    expect(screen.getByDisplayValue("C")).toBeInTheDocument();
    expect(screen.getByDisplayValue("Xyz")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Tag chord at caret" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  });
});
