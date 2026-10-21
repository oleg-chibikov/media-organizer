import type { MoveConfirmationSummary, MovePlan, MoveResult } from "../../lib/types";

export function countPendingExecutableMoves(
  plansByFileId: Record<string, MovePlan>,
  resultsByFileId: Record<string, MoveResult>
): number {
  return Object.values(plansByFileId).filter(
    (plan) => plan.plan_status === "ready" && resultsByFileId[plan.file_id] === undefined
  ).length;
}

export function buildMoveConfirmationPrompt(summary: MoveConfirmationSummary): string {
  return (
    `Ready operations: ${summary.ready_operations}\n` +
    `Skipped operations: ${summary.skipped_operations}\n` +
    `Fallback warnings: ${summary.fallback_warnings}\n` +
    `Suffix-applied destinations: ${summary.suffix_applied}\n\n` +
    "Proceed with move execution now?"
  );
}
