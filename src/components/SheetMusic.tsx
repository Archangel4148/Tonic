import { useEffect, useRef, useState } from "react";
import { OpenSheetMusicDisplay } from "opensheetmusicdisplay";

type Props = {
  xml: string;
  label?: string;
};

export function SheetMusic({ xml, label = "Sheet music" }: Props) {
  const hostRef = useRef<HTMLDivElement>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) {
      return;
    }
    host.replaceChildren();
    setError(null);
    const osmd = new OpenSheetMusicDisplay(host, {
      backend: "svg",
      autoResize: true,
      drawTitle: true,
      drawSubtitle: true,
      drawComposer: true,
      drawCredits: true,
      drawPartNames: false,
      drawMeasureNumbers: true,
    });
    let cancelled = false;
    osmd
      .load(xml)
      .then(() => {
        if (!cancelled) {
          osmd.render();
        }
      })
      .catch((cause: unknown) => {
        if (!cancelled) {
          setError(
            cause instanceof Error
              ? cause.message
              : "Sheet music could not be rendered.",
          );
        }
      });
    return () => {
      cancelled = true;
      host.replaceChildren();
    };
  }, [xml]);

  return (
    <div className="sheet-music" role="img" aria-label={label}>
      {error && (
        <p className="sheet-music-error" role="status">
          {error}
        </p>
      )}
      <div ref={hostRef} className="sheet-music-host" />
    </div>
  );
}
