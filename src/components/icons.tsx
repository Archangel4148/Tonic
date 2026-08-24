type IconProps = {
  className?: string;
};

const base = {
  width: "1.2em",
  height: "1.2em",
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.75,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
  "aria-hidden": true as const,
  focusable: false as const,
};

export function IconFullscreen({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <path d="M9 3H3v6" />
      <path d="M15 3h6v6" />
      <path d="M21 15v6h-6" />
      <path d="M3 15v6h6" />
    </svg>
  );
}

export function IconWindowed({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <path d="M9 3v6H3" />
      <path d="M15 3v6h6" />
      <path d="M21 15h-6v6" />
      <path d="M3 15h6v6" />
    </svg>
  );
}

export function IconLock({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <rect x="5" y="11" width="14" height="10" rx="2" />
      <path d="M8 11V8a4 4 0 0 1 8 0v3" />
    </svg>
  );
}

export function IconUnlock({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <rect x="5" y="11" width="14" height="10" rx="2" />
      <path d="M8 11V8a4 4 0 0 1 7.5-2" />
    </svg>
  );
}

export function IconExit({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
      <path d="M16 17l5-5-5-5" />
      <path d="M21 12H9" />
    </svg>
  );
}

export function IconChevronLeft({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <path d="M15 18l-6-6 6-6" />
    </svg>
  );
}

export function IconChevronRight({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <path d="M9 18l6-6-6-6" />
    </svg>
  );
}

export function IconPlay({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <path d="M8 5v14l11-7z" fill="currentColor" stroke="none" />
    </svg>
  );
}

export function IconPause({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <path d="M8 5h3v14H8zM13 5h3v14h-3z" fill="currentColor" stroke="none" />
    </svg>
  );
}

export function IconToTop({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <path d="M5 5h14" />
      <path d="M12 19V9" />
      <path d="M7 13l5-5 5 5" />
    </svg>
  );
}

export function IconEye({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  );
}

export function IconEyeOff({ className }: IconProps) {
  return (
    <svg {...base} className={className}>
      <path d="M3 3l18 18" />
      <path d="M10.6 10.6a3 3 0 0 0 4.2 4.2" />
      <path d="M9.9 5.1A10.7 10.7 0 0 1 12 5c6 0 10 7 10 7a17.4 17.4 0 0 1-3.2 4.1" />
      <path d="M6.1 6.1A17.5 17.5 0 0 0 2 12s4 7 10 7a10.5 10.5 0 0 0 4.3-.9" />
    </svg>
  );
}
