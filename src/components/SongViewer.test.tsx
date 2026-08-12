import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SongViewer } from "./SongViewer";
import type { SongSession } from "../lib/types";

const session: SongSession = {
  song: {
    id: "s1",
    title: "Demo",
    artist: null,
    album: null,
    originalKey: "C",
    performanceKey: "D",
    tempoBpm: null,
    timeSignature: null,
    notes: null,
    sourceFormat: "chordPro",
    hasScore: false,
    sections: [
      {
        label: "Chorus",
        lines: [
          {
            lyrics: "Hello world",
            chords: [
              {
                symbol: "D",
                written: "C",
                lyricIndex: 0,
                column: 0,
                status: "fullyRecognized",
              },
              {
                symbol: "Xyz",
                written: "Xyz",
                lyricIndex: 6,
                column: 6,
                status: "unrecognized",
              },
            ],
            annotations: ["quietly"],
          },
        ],
      },
    ],
  },
  warnings: [
    {
      kind: "unrecognizedChord",
      message: "Unrecognized chord 'Xyz' was preserved.",
      line: 2,
    },
  ],
  summaryMessage: "Some content could not be recognized.",
  semitoneOffset: 2,
  favorite: false,
  tags: ["demo"],
  setlist: null,
  sheetMusicXml: null,
};

describe("SongViewer", () => {
  it("shows section headers, lyrics, display chords, and warnings", () => {
    render(<SongViewer session={session} />);

    expect(screen.getByRole("heading", { name: "Demo" })).toBeInTheDocument();
    expect(screen.getByText("Chorus")).toBeInTheDocument();
    expect(screen.getByLabelText(/Hello world\. Chords:/i)).toBeInTheDocument();
    expect(screen.getByTitle("D (written C)")).toBeInTheDocument();
    expect(screen.getByTitle("Xyz (unrecognized)")).toBeInTheDocument();
    expect(screen.getByLabelText(/Xyz \(unrecognized\)/i)).toBeInTheDocument();
    expect(screen.getByText("quietly")).toBeInTheDocument();
    expect(
      screen.getByText("Some content could not be recognized."),
    ).toBeInTheDocument();
  });

  it("renders sheet music when derived MusicXML is present", () => {
    render(
      <SongViewer
        session={{
          ...session,
          song: { ...session.song, hasScore: true, sourceFormat: "musicXml" },
          sheetMusicXml: '<score-partwise version="4.0"></score-partwise>',
        }}
      />,
    );

    expect(screen.getByRole("img", { name: "Demo score" })).toBeInTheDocument();
  });
});
