import type { FileIdMap } from "../../lib/fileIdMap";
import { useCallback, type RefObject } from "react";
import { pauseMoves, resumeMoves, startMoves } from "../../lib/api";
import type { FileRecord, MetadataResult, MovePlan } from "../../lib/types";
import { IDLE_MOVE_EXECUTION } from "./moveExecutionState";
import { useMoveConfirmation } from "./useMoveConfirmation";
import { useMoveConfirmationSummary } from "./useMoveConfirmationSummary";
import { useMoveEvents } from "./useMoveEvents";
import { useMoveStatus } from "./useMoveStatus";

interface MoveExecutionOptions {
  // Checking files can still add ready plans, so a run may start before the first one.
  stillChecking: boolean;
  filesFound: boolean;
  selectedFolder: string | null;
  rows: FileRecord[];
  rowByIdRef: RefObject<Map<string, FileRecord>>;
  // The backend moves the files of this scan by their plans and cleans the source after.
  scanIdRef: RefObject<number | null>;
  metadataByFileId: FileIdMap<MetadataResult>;
  movePlansByFileId: FileIdMap<MovePlan>;
  reportError: (message: string) => void;
}

export function useMoveExecution(options: MoveExecutionOptions) {
  const { stillChecking, filesFound, selectedFolder, rowByIdRef, scanIdRef, reportError } = options;
  const { rows, metadataByFileId, movePlansByFileId } = options;
  const status = useMoveStatus();
  const { execution, moveResultsByFileId, updateExecution, clearStatus } = status;
  useMoveEvents({ scanIdRef, rowByIdRef, status, reportError });

  const { confirmationSummary, pendingExecutableCount } = useMoveConfirmationSummary(
    rows,
    metadataByFileId,
    movePlansByFileId,
    moveResultsByFileId
  );

  const canStart = pendingExecutableCount > 0 || (stillChecking && filesFound);

  const clearMoves = clearStatus;
  const startExecution = () => {
    const scanId = scanIdRef.current;
    if (!canStart || execution.confirmed) {
      return;
    }
    if (!selectedFolder || scanId === null) {
      reportError("No selected folder for move execution.");
      return;
    }
    const statusMessage = "Starting...";
    clearMoves(() => ({ ...IDLE_MOVE_EXECUTION, confirmed: true, executing: true, statusMessage }));
    startMoves(scanId).catch((error: unknown) => {
      reportError(`Failed to start moving: ${String(error)}`);
      clearMoves(() => IDLE_MOVE_EXECUTION);
    });
  };
  const confirmation = useMoveConfirmation({ ...options, onConfirmed: startExecution });

  const { cancel: cancelConfirmation } = confirmation;
  const resetMoves = useCallback(() => {
    cancelConfirmation();
    clearMoves(() => IDLE_MOVE_EXECUTION);
  }, [cancelConfirmation, clearMoves]);

  const togglePause = () => {
    const scanId = scanIdRef.current;
    if (!execution.confirmed || execution.completed || scanId === null) {
      return;
    }
    const paused = !execution.paused;
    updateExecution({ paused, statusMessage: paused ? "Paused." : "Resumed." });
    (paused ? pauseMoves : resumeMoves)(scanId).catch((error: unknown) =>
      reportError(`Failed to ${paused ? "pause" : "resume"}: ${String(error)}`)
    );
  };

  return {
    execution,
    moveResultsByFileId,
    moveCounts: status.moveCounts,
    confirmationSummary,
    canRunExecutionAction: execution.confirmed || canStart,
    confirmation: {
      summary: confirmation.summary,
      confirm: confirmation.confirm,
      cancel: confirmation.cancel
    },
    runExecutionAction: execution.confirmed ? togglePause : () => void confirmation.request(),
    resetMoves
  };
}
