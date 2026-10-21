import type { FileRecord, MoveResultStatus, PlanStatus } from "../../lib/types";
import type { FileRowState } from "./fileRowState";

const SCAN_STATUS_LABELS: Record<FileRecord["scan_status"], string> = {
  discovered: "Discovered",
  skipped_processed: "Skipped Processed",
  skipped_unsupported: "Skipped Unsupported",
  scan_error: "Scan Error"
};

const MOVE_RESULT_LABELS: Record<MoveResultStatus, string> = {
  moved: "Moved",
  copied: "Copied (source kept)",
  skipped: "Skipped",
  failed: "Failed"
};

const PLAN_STATUS_LABELS: Record<PlanStatus, string> = {
  ready: "Ready",
  blocked_no_date: "Blocked No Date",
  blocked_conflict: "Blocked Conflict",
  error: "Plan Error"
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
    return "Metadata Error";
  }
  if (planError && !movePlan) {
    return "Plan Error";
  }
  if (metadata?.date_kind === "fallback") {
    return "Warning Fallback";
  }
  if (!movePlan) {
    return metadata?.metadata_status === "ready" ? "Planning" : "Pending Metadata";
  }
  if (movePlan.plan_status === "ready" && movePlan.conflict_status === "suffix_applied") {
    return "Ready (Suffix)";
  }
  return PLAN_STATUS_LABELS[movePlan.plan_status] ?? "Pending Plan";
}
