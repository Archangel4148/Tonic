type Props = {
  originalKey: string | null;
  performanceKey: string | null;
  semitoneOffset: number;
  keys: string[];
  disabled?: boolean;
  onTranspose: (semitones: number) => void;
  onSelectKey: (key: string) => void;
  onReset: () => void;
};

export function TransposeBar({
  originalKey,
  performanceKey,
  semitoneOffset,
  keys,
  disabled = false,
  onTranspose,
  onSelectKey,
  onReset,
}: Props) {
  const selectValue = performanceKey ?? originalKey ?? "";

  return (
    <div className="transpose-bar" role="group" aria-label="Transpose">
      <button
        type="button"
        className="icon-button"
        onClick={() => onTranspose(-1)}
        disabled={disabled}
        aria-label="Transpose down a semitone"
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
        disabled={disabled}
        aria-label="Transpose up a semitone"
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
