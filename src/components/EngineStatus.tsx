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
            {info.persistenceHealthy ? "In-memory stub healthy" : "Unavailable"}
          </dd>
        </div>
      </dl>
    </details>
  );
}
