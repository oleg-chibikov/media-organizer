import { useCallback, useState } from "react";
import type { MoveResult } from "../../lib/types";
import { EMPTY_MOVE_COUNTS } from "./moveCounts";
import {
  IDLE_MOVE_EXECUTION,
  patchMoveExecution,
  type MoveExecutionState
} from "./moveExecutionState";

export type MoveStatus = ReturnType<typeof useMoveStatus>;

// Holds what the UI shows about moves: execution flags, per-file results and totals.
export function useMoveStatus() {
  const [execution, setExecution] = useState<MoveExecutionState>(IDLE_MOVE_EXECUTION);
  const [moveResultsByFileId, setMoveResultsByFileId] = useState<Record<string, MoveResult>>({});
  const [moveCounts, setMoveCounts] = useState(EMPTY_MOVE_COUNTS);

  const updateExecution = useCallback(
    (patch: Partial<MoveExecutionState>) =>
      setExecution((previous) => patchMoveExecution(previous, patch)),
    []
  );

  const clearStatus = useCallback(
    (nextExecution: (previous: MoveExecutionState) => MoveExecutionState) => {
      setExecution(nextExecution);
      setMoveResultsByFileId({});
      setMoveCounts(EMPTY_MOVE_COUNTS);
    },
    []
  );

  return {
    execution,
    setExecution,
    updateExecution,
    moveResultsByFileId,
    setMoveResultsByFileId,
    moveCounts,
    setMoveCounts,
    clearStatus
  };
}
