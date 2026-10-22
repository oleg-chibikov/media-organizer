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

// Mirrors the backend: the span is the suggested one widened by the real file dates,
// and by the files already placed when the suggestion continues a folder.
function foldersFor(base: string, fileIds: string[], spanDates: string[], dateOf: DateOf) {
  const countByDay = new Map<string, number>();
  for (const fileId of fileIds) {
    const date = dateOf(fileId);
    if (date) {
      const day = dayOf(date);
      countByDay.set(day, (countByDay.get(day) ?? 0) + 1);
    }
  }
  const days = [...countByDay.keys()].sort();
  const span = [...spanDates.map(dayOf), ...days].sort();
  const [first, last] = [span[0], span[span.length - 1]];
  const countByPath = new Map<string, number>();
  for (const day of days) {
    const folder = dayFolder(first, last, day);
    const path = folder ? `${base}/${folder}` : base;
    countByPath.set(path, (countByPath.get(path) ?? 0) + (countByDay.get(day) ?? 0));
  }
  return [...countByPath].map(([path, count]) => ({ path, count }));
}

// The folders the files of one suggestion go to, relative to the destination root.
// A continued suggestion keeps the placed year folder.
export function placeFolderPreview(suggestion: PlaceSuggestion, dateOf: DateOf): PreviewFolder[] {
  const { continues } = suggestion;
  const year = continues ? String(continues.year) : suggestion.start.slice(0, 4);
  const base = suggestion.kind === "trip" ? `${year}/${suggestion.title}` : year;
  const placedSpan = (name: string) => {
    const placed = continues?.places.find((place) => place.name === name);
    return placed ? [placed.start, placed.end] : [];
  };
  const places = suggestion.places.flatMap((place) =>
    foldersFor(
      `${base}/${place.name}`,
      place.file_ids,
      [place.start, place.end, ...placedSpan(place.name)],
      dateOf
    )
  );
  const rootFiles = suggestion.kind === "trip" ? suggestion.root_file_ids : [];
  const tripSpan = [suggestion.start, suggestion.end];
  const rootSpan = continues ? [...tripSpan, continues.start, continues.end] : tripSpan;
  return [...places, ...foldersFor(base, rootFiles, rootSpan, dateOf)];
}
