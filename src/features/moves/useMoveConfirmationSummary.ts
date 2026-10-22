import type { FileIdMap } from "../../lib/fileIdMap";
import { useMemo } from "react";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";
import { countPendingExecutableMoves } from "./moveConfirmationPrompt";
import { summarizeMoveConfirmation } from "./summarizeMoveConfirmation";

// Derives the live plan tally shown in the status bar and the confirmation prompt.
export function useMoveConfirmationSummary(
  rows: FileRecord[],
  metadataByFileId: FileIdMap<MetadataResult>,
  movePlansByFileId: FileIdMap<MovePlan>,
  moveResultsByFileId: FileIdMap<MoveResult>
) {
  const confirmationSummary = useMemo(
    () => summarizeMoveConfirmation(rows, metadataByFileId, movePlansByFileId),
    [metadataByFileId, movePlansByFileId, rows]
  );
  const pendingExecutableCount = useMemo(
    () => countPendingExecutableMoves(movePlansByFileId, moveResultsByFileId),
    [movePlansByFileId, moveResultsByFileId]
  );
  return { confirmationSummary, pendingExecutableCount };
}
