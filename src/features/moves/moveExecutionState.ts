export interface MoveExecutionState {
  confirmed: boolean;
  paused: boolean;
  executing: boolean;
  completed: boolean;
  activeFileId: string | null;
  statusMessage: string | null;
}

export const IDLE_MOVE_EXECUTION: MoveExecutionState = {
  confirmed: false,
  paused: false,
  executing: false,
  completed: false,
  activeFileId: null,
  statusMessage: null
};

export const FINISHED_RUN: Partial<MoveExecutionState> = {
  executing: false,
  activeFileId: null,
  statusMessage: null,
  completed: true
};

// Keeps the previous object when nothing changes so React can skip the render.
export function patchMoveExecution(
  previous: MoveExecutionState,
  patch: Partial<MoveExecutionState>
): MoveExecutionState {
  const keys = Object.keys(patch) as (keyof MoveExecutionState)[];
  return keys.some((key) => previous[key] !== patch[key]) ? { ...previous, ...patch } : previous;
}
