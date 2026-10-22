import type { FileIdMap } from "../../lib/fileIdMap";
import type {
  FileRecord,
  MetadataResult,
  MoveConfirmationSummary,
  MovePlan,
  MoveResult
} from "../../lib/types";

export function countPendingExecutableMoves(
  plansByFileId: FileIdMap<MovePlan>,
  resultsByFileId: FileIdMap<MoveResult>
): number {
  let count = 0;
  for (const plan of plansByFileId.values()) {
    if (plan.plan_status === "ready" && !resultsByFileId.has(plan.file_id)) {
      count += 1;
    }
  }
  return count;
}

// Every planned file of the current scan, so the backend can count what it would move or skip.
// The backend has each file's record and date, so only the plans go.
export function collectConfirmationPlans(
  rows: FileRecord[],
  metadataByFileId: FileIdMap<MetadataResult>,
  plansByFileId: FileIdMap<MovePlan>
): MovePlan[] {
  const plans: MovePlan[] = [];
  for (const row of rows) {
    const plan = plansByFileId.get(row.id);
    if (plan && metadataByFileId.has(row.id)) {
      plans.push(plan);
    }
  }
  return plans;
}

export function fileCountLabel(count: number): string {
  return count === 1 ? "1 file" : `${count} files`;
}

const sentence = (count: number, one: string, many: string) =>
  `${fileCountLabel(count)} ${count === 1 ? one : many}`;

// Only the parts of the summary that change what happens to some files.
export function moveConfirmationNotes(summary: MoveConfirmationSummary): string[] {
  const notes: string[] = [];
  if (summary.fallback_warnings > 0) {
    notes.push(
      sentence(
        summary.fallback_warnings,
        "uses the date it last changed. It has no capture date.",
        "use the date they last changed. They have no capture date."
      )
    );
  }
  if (summary.suffix_applied > 0) {
    notes.push(
      sentence(
        summary.suffix_applied,
        "gets a number added to its name. A file with that name is already there.",
        "get a number added to their names. Files with those names are already there."
      )
    );
  }
  if (summary.skipped_operations > 0) {
    notes.push(
      sentence(
        summary.skipped_operations,
        "stays where it is. See Needs review and Errors.",
        "stay where they are. See Needs review and Errors."
      )
    );
  }
  return notes;
}
