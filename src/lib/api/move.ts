import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  FileRecord,
  MetadataResult,
  MoveFileEvent,
  MoveFinishedEvent,
  MovePlan,
  MoveProgressEvent
} from "../types";
import { ensureTauriRuntime, invokeCommand } from "./tauriRuntime";

export const MOVE_EVENTS = {
  file: "move:file",
  progress: "move:progress",
  finished: "move:finished"
} as const;

export const executeMovePlan = (
  selectedPath: string,
  files: FileRecord[],
  metadataResults: MetadataResult[],
  plans: MovePlan[]
) =>
  invokeCommand<void>("execute_move_plan", {
    selectedPath,
    files,
    metadataResults,
    plans
  });

export interface MoveListeners {
  onProgress: (payload: MoveProgressEvent) => void;
  onFile: (payload: MoveFileEvent) => void;
  onFinished: (payload: MoveFinishedEvent) => void;
}

export async function attachMoveListeners(handlers: MoveListeners): Promise<UnlistenFn[]> {
  ensureTauriRuntime();

  const listeners = await Promise.all([
    listen<MoveProgressEvent>(MOVE_EVENTS.progress, (event) => handlers.onProgress(event.payload)),
    listen<MoveFileEvent>(MOVE_EVENTS.file, (event) => handlers.onFile(event.payload)),
    listen<MoveFinishedEvent>(MOVE_EVENTS.finished, (event) => handlers.onFinished(event.payload))
  ]);

  return listeners;
}
