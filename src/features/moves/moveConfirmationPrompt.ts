import type {
  FileRecord,
  MetadataResult,
  MoveConfirmationSummary,
  MovePlan,
  MoveResult
} from "../../lib/types";

export interface MoveConfirmationInput {
  files: FileRecord[];
  metadata: MetadataResult[];
  plans: MovePlan[];
}

export function countPendingExecutableMoves(
  plansByFileId: Record<string, MovePlan>,
  resultsByFileId: Record<string, MoveResult>
): number {
  return Object.values(plansByFileId).filter(
    (plan) => plan.plan_status === "ready" && resultsByFileId[plan.file_id] === undefined
  ).length;
}

// Every planned file of the current scan, so the backend can count what it would move or skip.
export function collectConfirmationInput(
  rows: FileRecord[],
  metadataByFileId: Record<string, MetadataResult>,
  plansByFileId: Record<string, MovePlan>
): MoveConfirmationInput {
  const input: MoveConfirmationInput = { files: [], metadata: [], plans: [] };
  for (const row of rows) {
    const metadata = metadataByFileId[row.id];
    const plan = plansByFileId[row.id];
    if (metadata && plan) {
      input.files.push(row);
      input.metadata.push(metadata);
      input.plans.push(plan);
    }
  }
  return input;
}

export function buildMoveConfirmationPrompt(summary: MoveConfirmationSummary): string {
  return (
    `Ready operations: ${summary.ready_operations}\n` +
    `Skipped operations: ${summary.skipped_operations}\n` +
    `Fallback warnings: ${summary.fallback_warnings}\n` +
    `Suffix-applied destinations: ${summary.suffix_applied}\n\n` +
    "Files in this folder that finish planning later are moved as well.\n" +
    "Proceed with move execution now?"
  );
}
