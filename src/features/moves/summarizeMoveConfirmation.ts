import type { FileIdMap } from "../../lib/fileIdMap";
import type {
  FileRecord,
  MetadataResult,
  MoveConfirmationSummary,
  MovePlan
} from "../../lib/types";

export function summarizeMoveConfirmation(
  rows: FileRecord[],
  metadataByFileId: FileIdMap<MetadataResult>,
  plansByFileId: FileIdMap<MovePlan>
): MoveConfirmationSummary {
  // Files that already have their planned name have nothing to do.
  const plans = [...plansByFileId.values()].filter((plan) => plan.plan_status !== "already_named");
  const rowsById = new Map(rows.map((row) => [row.id, row]));
  const summary: MoveConfirmationSummary = {
    total_plans: plans.length,
    ready_operations: 0,
    skipped_operations: 0,
    blocked_no_date: 0,
    blocked_conflict: 0,
    plan_errors: 0,
    suffix_applied: 0,
    fallback_warnings: 0,
    photos_ready: 0,
    videos_ready: 0
  };

  for (const plan of plans) {
    if (plan.plan_status === "blocked_no_date") {
      summary.blocked_no_date += 1;
    } else if (plan.plan_status === "blocked_conflict") {
      summary.blocked_conflict += 1;
    } else if (plan.plan_status === "error") {
      summary.plan_errors += 1;
    } else if (plan.plan_status === "ready") {
      summary.ready_operations += 1;
      if (plan.conflict_status === "suffix_applied") {
        summary.suffix_applied += 1;
      }
      if (metadataByFileId.get(plan.file_id)?.date_kind === "fallback") {
        summary.fallback_warnings += 1;
      }
      const kind = rowsById.get(plan.file_id)?.kind;
      if (kind === "photo") {
        summary.photos_ready += 1;
      } else if (kind === "video") {
        summary.videos_ready += 1;
      }
    }
  }

  summary.skipped_operations = plans.length - summary.ready_operations;
  return summary;
}
