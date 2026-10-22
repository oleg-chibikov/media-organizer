import type { MoveFileEvent, MoveFinishedEvent, MoveProgressEvent } from "../types";
import { listenAll } from "./listenAll";
import { invokeCommand } from "./tauriRuntime";

// Moves the files of scan `scanId` by their current plans in the backend.
export const executeMovePlan = (selectedPath: string, scanId: number, fileIds: string[]) =>
  invokeCommand<void>("execute_move_plan", { selectedPath, scanId, fileIds });

// Stops the running batch after the file moving now. Files it didn't reach get no result.
export const pauseMoveRun = () => invokeCommand<void>("pause_move_run");

// Resolves true when the selected folder itself was empty and got removed.
export const removeEmptySourceFolders = (selectedPath: string) =>
  invokeCommand<boolean>("remove_empty_source_folders", { selectedPath });

export interface MoveListeners {
  onProgress: (payload: MoveProgressEvent) => void;
  onFile: (payload: MoveFileEvent) => void;
  onFinished: (payload: MoveFinishedEvent) => void;
}

const MOVE_EVENTS = {
  onProgress: "move:progress",
  onFile: "move:file",
  onFinished: "move:finished"
} as const;

export const attachMoveListeners = (handlers: MoveListeners) => listenAll(MOVE_EVENTS, handlers);
