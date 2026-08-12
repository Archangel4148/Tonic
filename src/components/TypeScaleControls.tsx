import type { TypeScale } from "../lib/types";
import { clampScale } from "../lib/theme";

type Props = {
  scale: TypeScale;
  onChange: (scale: TypeScale) => void;
};

const STEP = 0.1;

export function TypeScaleControls({ scale, onChange }: Props) {
  return (
    <div className="type-scale" role="group" aria-label="Text size">
      <ScaleStepper
        label="Lyrics"
        value={scale.lyric}
        onChange={(lyric) => onChange({ ...scale, lyric })}
      />
      <ScaleStepper
        label="Chords"
        value={scale.chord}
        onChange={(chord) => onChange({ ...scale, chord })}
      />
      <ScaleStepper
        label="Headers"
        value={scale.section}
        onChange={(section) => onChange({ ...scale, section })}
      />
    </div>
  );
}

function ScaleStepper({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <div className="scale-stepper">
      <span>{label}</span>
      <button
        type="button"
        className="icon-button icon-button--small"
        aria-label={`Decrease ${label.toLowerCase()} size`}
        onClick={() => onChange(clampScale(value - STEP))}
      >
        −
      </button>
      <button
        type="button"
        className="icon-button icon-button--small"
        aria-label={`Increase ${label.toLowerCase()} size`}
        onClick={() => onChange(clampScale(value + STEP))}
      >
        +
      </button>
    </div>
  );
}
