import { splitChartLine } from "../lib/chart-line";
import type { ChordView, LineView } from "../lib/types";

type Props = {
  line: LineView;
};

export function ChartLine({ line }: Props) {
  const segments = splitChartLine(line.lyrics, line.chords);
  const hasLyrics = line.lyrics.length > 0;
  const chordOnly = !hasLyrics && line.chords.length > 0;
  const ariaLabel = buildAriaLabel(line);

  return (
    <div
      className={
        chordOnly ? "chart-line chart-line--chords-only" : "chart-line"
      }
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
        <p className="chart-chords-only" aria-hidden="true">
          {line.chords.map((chord) => (
            <ChordMark
              key={`${chord.symbol}-${chord.lyricIndex}`}
              chord={chord}
            />
          ))}
        </p>
      ) : (
        segments.length > 0 && (
          <p className="chart-syllables" aria-hidden="true">
            {segments.map((segment, index) => (
              <span key={`${segment.text}-${index}`} className="syllable">
                <span className="syllable-chords">
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

function buildAriaLabel(line: LineView): string | undefined {
  if (line.lyrics.length === 0 && line.chords.length === 0) {
    return undefined;
  }
  const chordText = line.chords
    .map((chord) => {
      if (chord.status === "unrecognized") {
        return `${chord.symbol} (unrecognized)`;
      }
      if (chord.status === "partiallyRecognized") {
        return `${chord.symbol} (partial)`;
      }
      return chord.symbol;
    })
    .join(", ");
  if (line.lyrics.length === 0) {
    return chordText || undefined;
  }
  if (!chordText) {
    return line.lyrics;
  }
  return `${line.lyrics}. Chords: ${chordText}`;
}

function ChordMark({ chord }: { chord: ChordView }) {
  const unusual =
    chord.status === "unrecognized" || chord.status === "partiallyRecognized";
  const statusLabel =
    chord.status === "unrecognized"
      ? "unrecognized"
      : chord.status === "partiallyRecognized"
        ? "partially recognized"
        : null;
  return (
    <span
      className={unusual ? `chord chord--${chord.status}` : "chord"}
      title={
        chord.symbol === chord.written
          ? statusLabel
            ? `${chord.symbol} (${statusLabel})`
            : chord.symbol
          : `${chord.symbol} (written ${chord.written})`
      }
    >
      {chord.symbol}
      {statusLabel && <span className="sr-only"> ({statusLabel})</span>}
    </span>
  );
}
