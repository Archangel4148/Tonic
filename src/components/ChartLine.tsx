import { splitChartLine } from "../lib/chart-line";
import type { ChordView, LineView } from "../lib/types";

type Props = {
  line: LineView;
};

export function ChartLine({ line }: Props) {
  const segments = splitChartLine(line.lyrics, line.chords);
  const hasLyrics = line.lyrics.length > 0;
  const chordOnly = !hasLyrics && line.chords.length > 0;
  const ariaLabel = hasLyrics
    ? line.lyrics
    : chordOnly
      ? line.chords.map((chord) => chord.symbol).join(" ")
      : undefined;

  return (
    <div
      className={chordOnly ? "chart-line chart-line--chords-only" : "chart-line"}
      aria-label={ariaLabel}
    >
      {line.annotations.map((annotation) => (
        <p
          key={annotation}
          className={
            annotation.toLowerCase().startsWith("capo")
              ? "chart-annotation chart-annotation--capo"
              : "chart-annotation"
          }
        >
          {annotation}
        </p>
      ))}

      {chordOnly ? (
        <p className="chart-chords-only">
          {line.chords.map((chord) => (
            <ChordMark key={`${chord.symbol}-${chord.lyricIndex}`} chord={chord} />
          ))}
        </p>
      ) : (
        segments.length > 0 && (
          <p className="chart-syllables" aria-hidden={hasLyrics}>
            {segments.map((segment, index) => (
              <span key={`${segment.text}-${index}`} className="syllable">
                <span
                  className="syllable-chords"
                  aria-hidden={!segment.chords.length}
                >
                  {segment.chords.length === 0 ? (
                    <span className="chord chord--empty">&nbsp;</span>
                  ) : (
                    segment.chords.map((chord) => (
                      <ChordMark
                        key={`${chord.symbol}-${chord.lyricIndex}-${chord.written}`}
                        chord={chord}
                      />
                    ))
                  )}
                </span>
                <span className="syllable-lyric">
                  {hasLyrics ? segment.text || "\u00a0" : "\u00a0"}
                </span>
              </span>
            ))}
          </p>
        )
      )}
    </div>
  );
}

function ChordMark({ chord }: { chord: ChordView }) {
  const unusual =
    chord.status === "unrecognized" || chord.status === "partiallyRecognized";
  return (
    <span
      className={unusual ? `chord chord--${chord.status}` : "chord"}
      title={
        chord.symbol === chord.written
          ? chord.symbol
          : `${chord.symbol} (written ${chord.written})`
      }
    >
      {chord.symbol}
    </span>
  );
}
