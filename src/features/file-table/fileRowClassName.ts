import { fileRowCategory } from "./fileRowCategory";
import type { FileRowState } from "./fileRowState";

// The stripe on the left marks the row without relying on the background alone.
const ERROR_ROW = "bg-rose-950/50 text-rose-100 shadow-[inset_3px_0_0_var(--color-rose-500)]";
const REVIEW_ROW = "bg-amber-950/40 text-amber-100 shadow-[inset_3px_0_0_var(--color-amber-400)]";

export function fileRowClassName(state: FileRowState, activeMoveFileId: string | null): string {
  const status = state.moveResult?.result_status;
  if (activeMoveFileId === state.record.id && !state.moveResult) {
    return "bg-cyan-950/40 text-cyan-100";
  }
  if (status === "moved") {
    return "bg-emerald-950/30 text-emerald-200";
  }
  if (status === "copied") {
    return "bg-sky-950/30 text-sky-200";
  }
  const category = fileRowCategory(state);
  return category === "error" ? ERROR_ROW : category === "review" ? REVIEW_ROW : "";
}
