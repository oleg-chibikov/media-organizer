import type { ProblemsFirst } from "./sortProblemsFirst";

export type FileFilter = "all" | "review" | "error";

// Problem rows come first, so each filter is one slice of the sorted rows.
export function fileFilterRange(
  filter: FileFilter,
  { states, errorCount, reviewCount }: ProblemsFirst
): [start: number, end: number] {
  if (filter === "error") {
    return [0, errorCount];
  }
  if (filter === "review") {
    return [errorCount, errorCount + reviewCount];
  }
  return [0, states.length];
}
