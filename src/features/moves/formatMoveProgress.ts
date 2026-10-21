import type { MoveProgressEvent } from "../../lib/types";

// The backend counts files per batch, so the run-wide count comes from the status bar.
export function formatMoveProgress(payload: MoveProgressEvent, rowLabel?: string): string {
  const itemLabel = rowLabel ?? payload.file_id;
  return itemLabel ? `${payload.message} ${itemLabel}` : payload.message;
}
