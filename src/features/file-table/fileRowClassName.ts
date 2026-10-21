import type { FileRowState } from "./fileRowState";

export function fileRowClassName(
  { record, metadata, moveResult }: FileRowState,
  activeMoveFileId: string | null
): string {
  const status = moveResult?.result_status;
  const isActiveMove = activeMoveFileId === record.id && !moveResult;
  const isMoveError = status === "skipped" || status === "failed";
  const isError = isMoveError || record.scan_status === "skipped_unsupported";
  const isFallbackWarning = metadata?.date_kind === "fallback";
  return `${status === "moved" ? "bg-emerald-950/30 text-emerald-200" : ""} ${status === "copied" ? "bg-sky-950/30 text-sky-200" : ""} ${isActiveMove ? "bg-cyan-950/30 text-cyan-100" : ""} ${isError ? "bg-rose-950/30 text-rose-200" : ""} ${isFallbackWarning ? "bg-amber-950/20" : ""}`;
}
