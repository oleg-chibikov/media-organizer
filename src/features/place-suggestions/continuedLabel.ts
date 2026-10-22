import type { ContinuedPlacement } from "../../lib/types";

// Matches the short dates in History, for example "Oct 28, 2025".
const DATE_FORMAT = new Intl.DateTimeFormat("en-US", {
  year: "numeric",
  month: "short",
  day: "numeric"
});

// Reads the day of a `chosen_date` as a local date, so no time zone shifts it.
function formatDay(date: string): string {
  const [year, month, day] = date.slice(0, 10).split("-").map(Number);
  return DATE_FORMAT.format(new Date(year ?? 0, (month ?? 1) - 1, day ?? 1));
}

export function continuedLabel(placed: ContinuedPlacement): string {
  return `Continues ${placed.title} (${formatDay(placed.start)}–${formatDay(placed.end)})`;
}

const files = (count: number) => `${count} file${count === 1 ? "" : "s"}`;

// One line per month folder a country moves up from, for example
// "Moves 2026/5/USA into 2026/USA (34 files)".
export function liftedLabels(placed: ContinuedPlacement): string[] {
  const { year, title } = placed;
  return placed.lifted.map(
    ({ month_folder: month, file_count: count }) =>
      `Moves ${year}/${month}/${title} into ${year}/${title} (${files(count)})`
  );
}
