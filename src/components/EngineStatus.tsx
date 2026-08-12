import type { AppInfo } from "../lib/types";

type Props = {
  info: AppInfo;
};

function phaseLabel(phase: number): string {
  switch (phase) {
    case 1:
      return "Foundation";
    case 2:
      return "Music engine";
    case 3:
      return "Song model";
    case 4:
      return "Import";
    case 5:
      return "Viewer";
    case 6:
      return "Library";
    case 7:
      return "Editor";
    case 8:
      return "Setlists";
    case 9:
      return "Live";
    case 10:
      return "Sheet music";
    case 11:
      return "Polish";
    default:
      return "";
  }
}

export function EngineStatus({ info }: Props) {
  return (
    <details className="engine-status">
      <summary>Engine</summary>
      <dl className="status-list">
        <div>
          <dt>Application</dt>
          <dd>
            {info.name} v{info.version}
          </dd>
        </div>
        <div>
          <dt>Phase</dt>
          <dd>
            {info.phase}
            {phaseLabel(info.phase) ? ` — ${phaseLabel(info.phase)}` : ""}
          </dd>
        </div>
        <div>
          <dt>Domain engine</dt>
          <dd>
            {info.domainEngine} v{info.domainVersion}
          </dd>
        </div>
        <div>
          <dt>Persistence</dt>
          <dd>
            {info.persistenceHealthy ? "Local library healthy" : "Unavailable"}
          </dd>
        </div>
      </dl>
    </details>
  );
}
