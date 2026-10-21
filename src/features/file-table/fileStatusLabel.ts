import type { FileRecord, MoveResultStatus, PlanStatus } from "../../lib/types";
import type { FileRowState } from "./fileRowState";

const SCAN_STATUS_LABELS: Record<FileRecord["scan_status"], string> = {
  discovered: "Found",
  skipped_processed: "Already organized",
  skipped_unsupported: "Not a photo or video",
  scan_error: "Can't read file"
};

const MOVE_RESULT_LABELS: Record<MoveResultStatus, string> = {
  moved: "Moved",
  copied: "Copied, original kept",
  skipped: "Skipped",
  failed: "Failed"
};

const PLAN_STATUS_LABELS: Record<PlanStatus, string> = {
  ready: "Ready",
  blocked_no_date: "No date",
  blocked_conflict: "Name taken",
  already_named: "Already named",
  error: "Can't plan"
};

export function fileStatusLabel(
  { record, metadata, movePlan, moveResult, planError }: FileRowState,
  activeMoveFileId: string | null
): string {
  if (moveResult) {
    return MOVE_RESULT_LABELS[moveResult.result_status] ?? "Processed";
  }
  if (record.scan_status !== "discovered") {
    return SCAN_STATUS_LABELS[record.scan_status];
  }
  if (activeMoveFileId === record.id) {
    return "Moving";
  }
  if (metadata?.metadata_status === "error") {
    return "Can't read date";
  }
  if (planError && !movePlan) {
    return "Can't plan";
  }
  if (metadata?.date_kind === "fallback") {
    return "Ready (file date)";
  }
  if (!movePlan) {
    return metadata?.metadata_status === "ready" ? "Planning" : "Reading date";
  }
  if (movePlan.plan_status === "ready" && movePlan.conflict_status === "suffix_applied") {
    return "Ready (renamed)";
  }
  return PLAN_STATUS_LABELS[movePlan.plan_status] ?? "Pending Plan";
}
