type Props = {
  fret: number;
  playedKey?: string | null;
  className?: string;
};

export function CapoBadge({ fret, playedKey, className }: Props) {
  const label = playedKey ? `Capo ${fret}, play ${playedKey} shapes` : `Capo ${fret}`;
  return (
    <span
      className={className ? `capo-badge ${className}` : "capo-badge"}
      title={label}
    >
      Capo {fret}
      {playedKey ? <span className="capo-badge-shapes">play {playedKey}</span> : null}
    </span>
  );
}
