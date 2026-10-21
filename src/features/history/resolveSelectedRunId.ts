import type { HistoryIndexEntry } from "../../lib/types";

// Keeps the current selection while it still exists, unless a caller prefers another run.
export function resolveSelectedRunId(
  previousRunId: string | null,
  preferredRunId: string | null,
  runs: HistoryIndexEntry[]
): string | null {
  const stillListed = previousRunId && runs.some((run) => run.run_id === previousRunId);
  return (preferredRunId ?? (stillListed ? previousRunId : null)) || null;
}
