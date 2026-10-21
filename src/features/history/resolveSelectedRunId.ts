import type { HistoryIndexEntry } from "../../lib/types";

// Picks the preferred run, else keeps the current one. A run missing from the list was deleted.
export function resolveSelectedRunId(
  previousRunId: string | null,
  preferredRunId: string | null,
  runs: HistoryIndexEntry[]
): string | null {
  const isListed = (runId: string | null) => runs.some((run) => run.run_id === runId);
  if (isListed(preferredRunId)) {
    return preferredRunId;
  }
  return isListed(previousRunId) ? previousRunId : null;
}
