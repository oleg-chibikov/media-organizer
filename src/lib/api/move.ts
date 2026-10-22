import type { MoveFileEvent, MoveFinishedEvent, MoveProgressEvent, MovesDoneEvent } from "../types";
import { listenAll } from "./listenAll";
import { invokeCommand } from "./tauriRuntime";

// Moves the scan's ready files, then the ones that become ready while it's still checked.
export const startMoves = (scanId: number) => invokeCommand<void>("start_moves", { scanId });

// Stops the run after the file moving now. Files it didn't reach get no result.
export const pauseMoves = (scanId: number) => invokeCommand<void>("pause_moves", { scanId });

export const resumeMoves = (scanId: number) => invokeCommand<void>("resume_moves", { scanId });

export interface MoveListeners {
  onProgress: (payload: MoveProgressEvent) => void;
  onFile: (payload: MoveFileEvent) => void;
  onFinished: (payload: MoveFinishedEvent) => void;
  onDone: (payload: MovesDoneEvent) => void;
}

const MOVE_EVENTS = {
  onProgress: "move:progress",
  onFile: "move:file",
  onFinished: "move:finished",
  onDone: "move:done"
} as const;

export const attachMoveListeners = (handlers: MoveListeners) => listenAll(MOVE_EVENTS, handlers);
