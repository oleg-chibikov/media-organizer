import type { FileRowState } from "./fileRowState";

// "error": the file failed and won't move. "review": it moves with a guess, or stays put on purpose.
export type FileRowCategory = "error" | "review" | "other";

export function fileRowCategory({
  record,
  metadata,
  movePlan,
  moveResult,
  planError
}: FileRowState): FileRowCategory {
  if (moveResult) {
    const notMoved =
      moveResult.result_status === "failed" || moveResult.result_status === "skipped";
    return notMoved ? "error" : "other";
  }
  if (
    record.scan_status === "scan_error" ||
    metadata?.metadata_status === "error" ||
    movePlan?.plan_status === "error" ||
    (planError !== undefined && !movePlan)
  ) {
    return "error";
  }
  if (
    record.scan_status === "skipped_unsupported" ||
    metadata?.metadata_status === "no_date" ||
    metadata?.date_kind === "fallback" ||
    (movePlan !== undefined && movePlan.plan_status !== "ready")
  ) {
    return "review";
  }
  return "other";
}
