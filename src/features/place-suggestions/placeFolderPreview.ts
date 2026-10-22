import type { PlaceSuggestion } from "../../lib/types";

export interface PreviewFolder {
  path: string;
  count: number;
}

type DateOf = (fileId: string) => string | null | undefined;

const DAY_MS = 86_400_000;
// Matches MONTH_DAY_MAX_SPAN_DAYS in src-tauri/src/trips/thresholds.rs.
const MONTH_DAY_MAX_SPAN_DAYS = 31;

const dayOf = (date: string) => date.slice(0, 10);

// One day gets no day folder, up to a month gets MM-DD, longer gets <month>/<day>.
function dayFolder(first: string, last: string, day: string): string | null {
  const spanDays = Math.round((Date.parse(last) - Date.parse(first)) / DAY_MS);
  if (spanDays === 0) {
    return null;
  }
  const [, month, date] = day.split("-");
  return spanDays <= MONTH_DAY_MAX_SPAN_DAYS
    ? `${month}-${date}`
    : `${Number(month)}/${Number(date)}`;
}

// Mirrors the backend: the span is the suggested one widened by the real file dates.
function foldersFor(base: string, fileIds: string[], start: string, end: string, dateOf: DateOf) {
  const countByDay = new Map<string, number>();
  for (const fileId of fileIds) {
    const date = dateOf(fileId);
    if (date) {
      const day = dayOf(date);
      countByDay.set(day, (countByDay.get(day) ?? 0) + 1);
    }
  }
  const days = [...countByDay.keys()].sort();
  const first = [dayOf(start), ...days].sort()[0];
  const last = [dayOf(end), ...days].sort().at(-1) ?? first;
  const countByPath = new Map<string, number>();
  for (const day of days) {
    const folder = dayFolder(first, last, day);
    const path = folder ? `${base}/${folder}` : base;
    countByPath.set(path, (countByPath.get(path) ?? 0) + (countByDay.get(day) ?? 0));
  }
  return [...countByPath].map(([path, count]) => ({ path, count }));
}

// The folders the files of one suggestion go to, relative to the destination root.
export function placeFolderPreview(suggestion: PlaceSuggestion, dateOf: DateOf): PreviewFolder[] {
  const year = suggestion.start.slice(0, 4);
  const base = suggestion.kind === "trip" ? `${year}/${suggestion.title}` : year;
  const places = suggestion.places.flatMap((place) =>
    foldersFor(`${base}/${place.name}`, place.file_ids, place.start, place.end, dateOf)
  );
  const rootFiles = suggestion.kind === "trip" ? suggestion.root_file_ids : [];
  return [...places, ...foldersFor(base, rootFiles, suggestion.start, suggestion.end, dateOf)];
}
