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
