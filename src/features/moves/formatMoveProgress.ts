import type { MoveProgressEvent } from "../../lib/types";

export function formatMoveProgress(payload: MoveProgressEvent, rowLabel?: string): string {
  const itemLabel = rowLabel ?? payload.file_id;
  const ordinal =
    payload.file_id && payload.total > 0
      ? ` ${Math.min(payload.completed + 1, payload.total)}/${payload.total}`
      : payload.total > 0
        ? ` ${payload.completed}/${payload.total}`
        : "";
  const detail = itemLabel ? ` ${itemLabel}` : "";
  return `${payload.message}${ordinal}${detail}`;
}
