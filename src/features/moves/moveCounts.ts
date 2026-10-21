import type { MoveResultStatus } from "../../lib/types";

export type MoveCounts = Record<MoveResultStatus, number>;

export const EMPTY_MOVE_COUNTS: MoveCounts = { moved: 0, copied: 0, skipped: 0, failed: 0 };

export function countMoveResult(counts: MoveCounts, status: MoveResultStatus): MoveCounts {
  return { ...counts, [status]: counts[status] + 1 };
}

export function hasMoveResults(counts: MoveCounts): boolean {
  return counts.moved + counts.copied + counts.skipped + counts.failed > 0;
}
