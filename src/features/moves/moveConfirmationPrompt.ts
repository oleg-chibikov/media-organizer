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

export function fileCountLabel(count: number): string {
  return count === 1 ? "1 file" : `${count} files`;
}

// Only the parts of the summary that change what happens to some files.
export function moveConfirmationNotes(summary: MoveConfirmationSummary): string[] {
  const notes: string[] = [];
  if (summary.fallback_warnings > 0) {
    notes.push(
      `${fileCountLabel(summary.fallback_warnings)} use the date the file last changed. They have no capture date.`
    );
  }
  if (summary.suffix_applied > 0) {
    notes.push(
      `${fileCountLabel(summary.suffix_applied)} get a number added to the name. A file with that name is already there.`
    );
  }
  if (summary.skipped_operations > 0) {
    notes.push(
      `${fileCountLabel(summary.skipped_operations)} stay where they are. See Needs review and Errors.`
    );
  }
  return notes;
}
