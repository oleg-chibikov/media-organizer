import type { MoveResult, MoveResultStatus } from "../../lib/types";

export type MoveCounts = Record<MoveResultStatus, number>;

// Tallies the latest result of each file, so repeated results for one file count once.
export function countMoveResults(resultsByFileId: Record<string, MoveResult>): MoveCounts {
  const counts: MoveCounts = { moved: 0, copied: 0, skipped: 0, failed: 0 };
  for (const result of Object.values(resultsByFileId)) {
    counts[result.result_status] += 1;
  }
  return counts;
}

export function hasMoveResults(counts: MoveCounts): boolean {
  return counts.moved + counts.copied + counts.skipped + counts.failed > 0;
}
