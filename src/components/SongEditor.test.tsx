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
  chartText: "[Verse]\nC       G\nHello world\n",
};

describe("SongEditor", () => {
  it("shows a plaintext chart and save/cancel", () => {
    render(
      <SongEditor
        editor={editor}
        keys={["C", "G"]}
        onChange={vi.fn()}
        onSave={vi.fn(async () => undefined)}
        onCancel={vi.fn()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "New song" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Chord and lyric chart")).toHaveValue(
      "[Verse]\nC       G\nHello world\n",
    );
    expect(
      screen.queryByRole("button", { name: "Tag chord at caret" }),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  });
});
