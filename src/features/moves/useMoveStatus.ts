import { useCallback, useMemo, useState } from "react";
import { FileIdMap } from "../../lib/fileIdMap";
import type { MoveResult } from "../../lib/types";
import { countMoveResults } from "./moveCounts";
import {
  IDLE_MOVE_EXECUTION,
  patchMoveExecution,
  type MoveExecutionState
} from "./moveExecutionState";

export type MoveStatus = ReturnType<typeof useMoveStatus>;

// Holds what the UI shows about moves: execution flags and the latest result per file.
export function useMoveStatus() {
  const [execution, setExecution] = useState<MoveExecutionState>(IDLE_MOVE_EXECUTION);
  const [moveResultsByFileId, setMoveResultsByFileId] = useState(FileIdMap.empty<MoveResult>);
  const moveCounts = useMemo(() => countMoveResults(moveResultsByFileId), [moveResultsByFileId]);

  const updateExecution = useCallback(
    (patch: Partial<MoveExecutionState>) =>
      setExecution((previous) => patchMoveExecution(previous, patch)),
    []
  );

  const recordResults = useCallback((results: MoveResult[]) => {
    if (results.length === 0) {
      return;
    }
    setMoveResultsByFileId((previous) => previous.with(results, (result) => result.file_id));
    const finishedIds = new Set(results.map((result) => result.file_id));
    setExecution((previous) =>
      previous.activeFileId && finishedIds.has(previous.activeFileId)
        ? patchMoveExecution(previous, { activeFileId: null })
        : previous
    );
  }, []);

  const clearStatus = useCallback(
    (nextExecution: (previous: MoveExecutionState) => MoveExecutionState) => {
      setExecution(nextExecution);
      setMoveResultsByFileId(FileIdMap.empty());
    },
    []
  );

  return {
    execution,
    moveResultsByFileId,
    moveCounts,
    updateExecution,
    recordResults,
    clearStatus
  };
}
