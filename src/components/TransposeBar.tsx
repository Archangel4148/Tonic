import type { TransposeMode } from "../lib/types";

type Props = {
  displayKey: string | null;
  originalKey?: string | null;
  playedKey?: string | null;
  semitoneOffset: number;
  keys: string[];
  mode: TransposeMode;
  capoFret: number | null;
  disabled?: boolean;
  onTranspose: (semitones: number) => void;
  onSelectKey: (key: string) => void;
  onSelectShapesKey?: (key: string) => void;
  onReset: () => void;
  onModeChange: (mode: TransposeMode) => void;
};

export function TransposeModeToggle({
  mode,
  disabled = false,
  onChange,
}: {
  mode: TransposeMode;
  disabled?: boolean;
  onChange: (mode: TransposeMode) => void;
}) {
  return (
    <div className="transpose-mode" role="group" aria-label="How to transpose">
      <button
        type="button"
        className="text-button"
        aria-pressed={mode === "chords"}
        disabled={disabled}
        aria-label="New chords"
        title="Rewrite chord names into the new key"
        onClick={() => onChange("chords")}
      >
        Chords
      </button>
      <button
        type="button"
        className="text-button"
        aria-pressed={mode === "capo"}
        disabled={disabled}
        title="Choose play shapes and move a capo for the sounding key"
        onClick={() => onChange("capo")}
      >
        Capo
      </button>
    </div>
  );
}

function KeySelect({
  label,
  value,
  keys,
  disabled,
  onChange,
}: {
  label: string;
  value: string;
  keys: string[];
  disabled?: boolean;
  onChange: (key: string) => void;
}) {
  return (
    <label className="key-select-label">
      <span className="key-select-caption">{label}</span>
      <select
        value={value}
        onChange={(event) => onChange(event.target.value)}
        disabled={disabled || keys.length === 0}
        aria-label={label}
      >
        {!value && <option value="">Key</option>}
        {value && !keys.includes(value) && <option value={value}>{value}</option>}
        {keys.map((key) => (
          <option key={key} value={key}>
            {key}
          </option>
        ))}
      </select>
    </label>
  );
}

export function TransposeBar({
  displayKey,
  originalKey = null,
  playedKey = null,
  semitoneOffset,
  keys,
  mode,
  capoFret,
  disabled = false,
  onTranspose,
  onSelectKey,
  onSelectShapesKey,
  onReset,
  onModeChange,
}: Props) {
  const selectValue = displayKey ?? "";
  const shapesValue = playedKey ?? originalKey ?? "";
  const fret = capoFret ?? 0;
  const showPlayAndSound = mode === "capo" && Boolean(onSelectShapesKey);
  const shapesCustom =
    mode === "capo" &&
    shapesValue.length > 0 &&
    originalKey != null &&
    shapesValue !== originalKey;
  const resetBlocked =
    mode === "capo" ? fret === 0 && !shapesCustom : semitoneOffset === 0;

  return (
    <div className="transpose-bar" role="group" aria-label="Transpose">
      <TransposeModeToggle
        mode={mode}
        disabled={disabled}
        onChange={onModeChange}
      />
      {showPlayAndSound && onSelectShapesKey && (
        <KeySelect
          label="Play"
          value={shapesValue}
          keys={keys}
          disabled={disabled}
          onChange={onSelectShapesKey}
        />
      )}
      {!showPlayAndSound && (
        <button
          type="button"
          className="icon-button"
          onClick={() => onTranspose(-1)}
          disabled={disabled}
          aria-label="Transpose down a semitone"
        >
          −
        </button>
      )}
      <KeySelect
        label={showPlayAndSound ? "Sound" : "Key"}
        value={selectValue}
        keys={keys}
        disabled={disabled}
        onChange={onSelectKey}
      />
      {!showPlayAndSound && (
        <button
          type="button"
          className="icon-button"
          onClick={() => onTranspose(1)}
          disabled={disabled}
          aria-label="Transpose up a semitone"
        >
          +
        </button>
      )}
      <button
        type="button"
        className="text-button"
        onClick={onReset}
        disabled={disabled || resetBlocked}
      >
        Reset
      </button>
    </div>
  );
}
