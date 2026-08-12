import { useEffect, useRef, useState } from "react";

type Props = {
  xml: string;
  label?: string;
};

type OsmdModule = typeof import("opensheetmusicdisplay");

export function SheetMusic({ xml, label = "Sheet music" }: Props) {
  const hostRef = useRef<HTMLDivElement>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) {
      return;
    }
    host.replaceChildren();
    setError(null);
    setLoading(true);
    let cancelled = false;

    void (async () => {
      try {
        const { OpenSheetMusicDisplay }: OsmdModule =
          await import("opensheetmusicdisplay");
        if (cancelled || !hostRef.current) {
          return;
        }
        const osmd = new OpenSheetMusicDisplay(hostRef.current, {
          backend: "svg",
          autoResize: true,
          drawTitle: true,
          drawSubtitle: true,
          drawComposer: true,
          drawCredits: true,
          drawPartNames: false,
          drawMeasureNumbers: true,
        });
        await osmd.load(xml);
        if (cancelled) {
          return;
        }
        osmd.render();
        setLoading(false);
      } catch (cause: unknown) {
        if (!cancelled) {
          setLoading(false);
          setError(
            cause instanceof Error
              ? cause.message
              : "Sheet music could not be rendered.",
          );
        }
      }
    })();

    return () => {
      cancelled = true;
      host.replaceChildren();
    };
  }, [xml]);

  return (
    <div
      className="sheet-music"
      role="img"
      aria-label={label}
      aria-busy={loading}
    >
      {loading && !error && (
        <p className="sheet-music-status" role="status">
          Rendering sheet music…
        </p>
      )}
      {error && (
        <p className="sheet-music-error" role="status">
          {error}
        </p>
      )}
      <div ref={hostRef} className="sheet-music-host" />
    </div>
  );
}
