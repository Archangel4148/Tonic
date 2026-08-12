import { useRef } from "react";
import { SAMPLES } from "../lib/samples";
import type { ImportFormat } from "../lib/types";

type Props = {
  text: string;
  format: ImportFormat;
  busy: boolean;
  onTextChange: (text: string) => void;
  onFormatChange: (format: ImportFormat) => void;
  onImport: (text: string, format: ImportFormat) => void;
};

export function ImportPanel({
  text,
  format,
  busy,
  onTextChange,
  onFormatChange,
  onImport,
}: Props) {
  const fileRef = useRef<HTMLInputElement>(null);

  return (
    <section className="import-panel" aria-labelledby="import-heading">
      <div className="import-panel-header">
        <h2 id="import-heading">Import</h2>
        <p className="hint">Paste a ChordPro or chord-over-lyrics chart.</p>
      </div>

      <div className="sample-row">
        {SAMPLES.map((sample) => (
          <button
            key={sample.id}
            type="button"
            className="chip"
            onClick={() => onTextChange(sample.text)}
          >
            {sample.label}
          </button>
        ))}
      </div>

      <label className="field-label" htmlFor="chart-text">
        Chart text
      </label>
      <textarea
        id="chart-text"
        value={text}
        onChange={(event) => onTextChange(event.target.value)}
        spellCheck={false}
        rows={10}
        placeholder="{title: Song}\n[C]Hello [G]world"
      />

      <div className="import-actions">
        <label className="field-label" htmlFor="import-format">
          Format
          <select
            id="import-format"
            value={format}
            onChange={(event) =>
              onFormatChange(event.target.value as ImportFormat)
            }
          >
            <option value="auto">Auto detect</option>
            <option value="chordPro">ChordPro</option>
            <option value="plainText">Plain text</option>
          </select>
        </label>

        <input
          ref={fileRef}
          type="file"
          accept=".cho,.crd,.chopro,.chordpro,.pro,.txt,.text"
          hidden
          onChange={async (event) => {
            const file = event.target.files?.[0];
            event.target.value = "";
            if (!file) {
              return;
            }
            const contents = await file.text();
            const ext = file.name.split(".").pop()?.toLowerCase();
            const nextFormat: ImportFormat =
              ext && ["cho", "crd", "chopro", "chordpro", "pro"].includes(ext)
                ? "chordPro"
                : ext === "txt" || ext === "text"
                  ? "plainText"
                  : format;
            onTextChange(contents);
            onFormatChange(nextFormat);
            onImport(contents, nextFormat);
          }}
        />

        <button
          type="button"
          className="text-button"
          onClick={() => fileRef.current?.click()}
        >
          Open file
        </button>

        <button
          type="button"
          className="primary-button"
          onClick={() => onImport(text, format)}
          disabled={busy || text.trim().length === 0}
        >
          {busy ? "Importing…" : "Import song"}
        </button>
      </div>
    </section>
  );
}
