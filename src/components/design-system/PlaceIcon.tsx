const PATHS = {
  home: "M2 7.5 8 2.5l6 5M3.5 6.5V13.5h3.5V10h2v3.5h3.5V6.5",
  away: "M14 6.5c.6 0 1 .5 1 1s-.4 1-1 1H10l-3 5H5.5l1.5-5H4L2.8 10H1.5l.7-2.5-.7-2.5h1.3L4 6.5h3l-1.5-5H7l3 5Z"
} as const;

const LABELS = { home: "Home", away: "Away from home" } as const;

// A house for a place that matches home, a plane for one away from it.
export function PlaceIcon({ kind }: { kind: keyof typeof PATHS }) {
  return (
    <svg
      role="img"
      aria-label={LABELS[kind]}
      viewBox="0 0 16 16"
      className={`mr-1.5 inline-block h-3.5 w-3.5 align-[-2px] ${kind === "home" ? "text-emerald-300" : "text-sky-300"}`}
      fill={kind === "home" ? "none" : "currentColor"}
      stroke={kind === "home" ? "currentColor" : "none"}
      strokeWidth="1.5"
      strokeLinejoin="round"
    >
      <title>{LABELS[kind]}</title>
      <path d={PATHS[kind]} />
    </svg>
  );
}
