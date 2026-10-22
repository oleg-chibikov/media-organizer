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

export interface FileStatus {
  label: string;
  // The file is still being read, planned or moved.
  busy?: boolean;
}

const busy = (label: string): FileStatus => ({ label, busy: true });

export function fileStatus(
  { record, metadata, movePlan, moveResult, planError }: FileRowState,
  activeMoveFileId: string | null
): FileStatus {
  if (moveResult) {
    return { label: MOVE_RESULT_LABELS[moveResult.result_status] ?? "Processed" };
  }
  if (record.scan_status !== "discovered") {
    return { label: SCAN_STATUS_LABELS[record.scan_status] };
  }
  if (activeMoveFileId === record.id) {
    return busy("Moving");
  }
  if (metadata?.metadata_status === "error") {
    return { label: "Can't read metadata" };
  }
  if (planError && !movePlan) {
    return { label: "Can't plan" };
  }
  if (metadata?.date_kind === "fallback") {
    return { label: "Ready (file date)" };
  }
  if (!movePlan) {
    return busy(metadata?.metadata_status === "ready" ? "Planning" : "Reading metadata");
  }
  if (movePlan.plan_status === "ready" && movePlan.conflict_status === "suffix_applied") {
    return { label: "Ready (renamed)" };
  }
  return { label: PLAN_STATUS_LABELS[movePlan.plan_status] ?? "Pending Plan" };
}
