import { useRef, useState } from "react";
import type { ImportFormat } from "../lib/types";

type Props = {
  text: string;
  format: ImportFormat;
  busy: boolean;
  onTextChange: (text: string) => void;
  onFormatChange: (format: ImportFormat) => void;
  onImport: (text: string, format: ImportFormat) => void;
  onImportBinary: (bytes: Uint8Array, fileName: string) => void;
  onImportUrl: (url: string) => void;
};

const CHART_EXTENSIONS = ["cho", "crd", "chopro", "chordpro", "pro"];

export function ImportPanel({
  text,
  format,
  busy,
  onTextChange,
  onFormatChange,
  onImport,
  onImportBinary,
  onImportUrl,
}: Props) {
  const fileRef = useRef<HTMLInputElement>(null);
  const [url, setUrl] = useState("");

  return (
    <section
      id="import-panel"
      className="import-panel"
      aria-labelledby="import-heading"
    >
      <div className="import-panel-header">
        <h2 id="import-heading">Import</h2>
        <p className="hint">
          Paste an Ultimate Guitar chords URL, paste chart text, or open a file.
          Imported songs stay on this device offline.
        </p>
      </div>

      <label className="field-label" htmlFor="import-url">
        Song URL
      </label>
      <div className="import-url-row">
        <input
          id="import-url"
          type="url"
          inputMode="url"
          autoCapitalize="off"
          autoCorrect="off"
          spellCheck={false}
          placeholder="https://tabs.ultimate-guitar.com/tab/…/…-chords-…"
          value={url}
          disabled={busy}
          onChange={(event) => setUrl(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && url.trim() && !busy) {
              event.preventDefault();
              onImportUrl(url.trim());
            }
          }}
        />
        <button
          type="button"
          className="primary-button"
          disabled={busy || url.trim().length === 0}
          onClick={() => onImportUrl(url.trim())}
        >
          {busy ? "Importing…" : "Import URL"}
        </button>
      </div>
      <p className="hint import-url-hint">
        Supported: Ultimate Guitar chord tabs
      </p>

      <label className="field-label" htmlFor="chart-text">
        Or paste chart text
      </label>
      <textarea
        id="chart-text"
        value={text}
        onChange={(event) => onTextChange(event.target.value)}
        spellCheck={false}
        rows={8}
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
            <option value="musicXml">MusicXML</option>
          </select>
        </label>

        {/* Android greys out unknown extensions when accept is extension-only. */}
        <input
          ref={fileRef}
          type="file"
          accept="*/*"
          hidden
          onChange={async (event) => {
            const file = event.target.files?.[0];
            event.target.value = "";
            if (!file) {
              return;
            }
            const ext = file.name.split(".").pop()?.toLowerCase() ?? "";
            if (ext === "mxl") {
              const bytes = new Uint8Array(await file.arrayBuffer());
              onImportBinary(bytes, file.name);
              return;
            }
            const contents = await file.text();
            const nextFormat: ImportFormat = CHART_EXTENSIONS.includes(ext)
              ? "chordPro"
              : ext === "txt" || ext === "text"
                ? "plainText"
                : ext === "musicxml" || ext === "xml"
                  ? "musicXml"
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
          className="text-button"
          onClick={() => onImport(text, format)}
          disabled={busy || text.trim().length === 0}
        >
          {busy ? "Importing…" : "Import text"}
        </button>
      </div>
    </section>
  );
}
