import type { MoveConfirmationSummary } from "../../lib/types";
import type { MoveCounts } from "../moves";

// Files with a result out of every found file that isn't blocked, including ones still checked.
export function moveProgressCount(
  discovered: number,
  planErrors: number,
  summary: MoveConfirmationSummary,
  moveCounts: MoveCounts
): string {
  const done = moveCounts.moved + moveCounts.copied + moveCounts.skipped + moveCounts.failed;
  const total = discovered - summary.skipped_operations - planErrors;
  return `${done}/${Math.max(done, total)}`;
}
