const ICONS = {
  home: {
    label: "Home",
    path: "M2 7.5 8 2.5l6 5M3.5 6.5V13.5h3.5V10h2v3.5h3.5V6.5",
    className: "fill-none stroke-current text-emerald-300"
  },
  road: {
    label: "In the home country",
    path: "M3 7 4.3 3.8C4.5 3.3 5 3 5.5 3h5c.5 0 1 .3 1.2.8L13 7c.6.1 1 .6 1 1.2V10.5c0 .3-.2.5-.5.5h-11a.5.5 0 0 1-.5-.5V8.2C2 7.6 2.4 7.1 3 7Zm1.6-.2h6.8l-.9-2.5H5.5ZM3.6 12.6a1.4 1.4 0 1 0 2.8 0 1.4 1.4 0 1 0-2.8 0Zm6 0a1.4 1.4 0 1 0 2.8 0 1.4 1.4 0 1 0-2.8 0Z",
    className: "fill-current text-amber-300"
  },
  away: {
    label: "Away from home",
    path: "M14 6.5c.6 0 1 .5 1 1s-.4 1-1 1H10l-3 5H5.5l1.5-5H4L2.8 10H1.5l.7-2.5-.7-2.5h1.3L4 6.5h3l-1.5-5H7l3 5Z",
    className: "fill-current text-sky-300"
  }
} as const;

export type PlaceIconKind = keyof typeof ICONS;

// A house for home, a car for elsewhere in the home country, a plane for abroad.
export function PlaceIcon({ kind }: { kind: PlaceIconKind }) {
  const { label, path, className } = ICONS[kind];
  return (
    <svg
      role="img"
      aria-label={label}
      viewBox="0 0 16 16"
      className={`mr-1.5 inline-block h-3.5 w-3.5 align-[-2px] ${className}`}
      fillRule="evenodd"
      strokeWidth="1.5"
      strokeLinejoin="round"
    >
      <title>{label}</title>
      <path d={path} />
    </svg>
  );
}
