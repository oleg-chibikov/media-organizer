import type { FileFilter } from "./fileFilter";

interface FileFilterBarProps {
  value: FileFilter;
  total: number;
  reviewCount: number;
  errorCount: number;
  onChange: (filter: FileFilter) => void;
}

const BASE = "cursor-pointer rounded px-2.5 py-1 text-xs font-medium";
const PRESSED = "bg-slate-700 text-white";
const UNPRESSED = "text-slate-300 hover:bg-slate-800";

export function FileFilterBar({
  value,
  total,
  reviewCount,
  errorCount,
  onChange
}: FileFilterBarProps) {
  const options: { filter: FileFilter; label: string; count: number; countClass: string }[] = [
    { filter: "all", label: "All", count: total, countClass: "text-slate-400" },
    {
      filter: "review",
      label: "Needs review",
      count: reviewCount,
      countClass: reviewCount > 0 ? "text-amber-300" : "text-slate-400"
    },
    {
      filter: "error",
      label: "Errors",
      count: errorCount,
      countClass: errorCount > 0 ? "text-rose-300" : "text-slate-400"
    }
  ];

  return (
    <div
      role="group"
      aria-label="Show files"
      className="flex gap-1 border-b border-slate-800 bg-slate-950 px-3 py-1.5"
    >
      {options.map(({ filter, label, count, countClass }) => (
        <button
          key={filter}
          type="button"
          aria-pressed={value === filter}
          className={`${BASE} ${value === filter ? PRESSED : UNPRESSED}`}
          onClick={() => onChange(filter)}
        >
          {label} <span className={countClass}>{count}</span>
        </button>
      ))}
    </div>
  );
}
