import type { TransposeMode } from "../lib/types";

type Props = {
  displayKey: string | null;
  semitoneOffset: number;
  keys: string[];
  mode: TransposeMode;
  capoFret: number | null;
  disabled?: boolean;
  onTranspose: (semitones: number) => void;
  onSelectKey: (key: string) => void;
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
        title="Rewrite chord names into the new key"
        onClick={() => onChange("chords")}
      >
        New chords
      </button>
      <button
        type="button"
        className="text-button"
        aria-pressed={mode === "capo"}
        disabled={disabled}
        title="Keep these chord shapes and move a capo"
        onClick={() => onChange("capo")}
      >
        Capo
      </button>
    </div>
  );
}

export function TransposeBar({
  displayKey,
  semitoneOffset,
  keys,
  mode,
  capoFret,
  disabled = false,
  onTranspose,
  onSelectKey,
  onReset,
  onModeChange,
}: Props) {
  const selectValue = displayKey ?? "";
  const fret = capoFret ?? 0;
  const capoDownBlocked = mode === "capo" && fret <= 0;
  const capoUpBlocked = mode === "capo" && fret >= 12;

  return (
    <div className="transpose-bar" role="group" aria-label="Transpose">
      <TransposeModeToggle
        mode={mode}
        disabled={disabled}
        onChange={onModeChange}
      />
      <button
        type="button"
        className="icon-button"
        onClick={() => onTranspose(-1)}
        disabled={disabled || capoDownBlocked}
        aria-label={
          mode === "capo"
            ? "Move capo down a fret"
            : "Transpose down a semitone"
        }
      >
        −
      </button>
      <label className="key-select-label">
        <span className="sr-only">Performance key</span>
        <select
          value={selectValue}
          onChange={(event) => onSelectKey(event.target.value)}
          disabled={disabled || keys.length === 0}
        >
          {!selectValue && <option value="">Key</option>}
          {selectValue && !keys.includes(selectValue) && (
            <option value={selectValue}>{selectValue}</option>
          )}
          {keys.map((key) => (
            <option key={key} value={key}>
              {key}
            </option>
          ))}
        </select>
      </label>
      <button
        type="button"
        className="icon-button"
        onClick={() => onTranspose(1)}
        disabled={disabled || capoUpBlocked}
        aria-label={
          mode === "capo" ? "Move capo up a fret" : "Transpose up a semitone"
        }
      >
        +
      </button>
      <button
        type="button"
        className="text-button"
        onClick={onReset}
        disabled={disabled || semitoneOffset === 0}
      >
        Reset
      </button>
    </div>
  );
}
