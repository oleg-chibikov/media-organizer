import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  HistoryIndexEntry,
  HistoryIndexUpdatedEvent,
  HistoryRedoProgressEvent,
  HistoryRunDetails,
  HistoryUndoProgressEvent
} from "../types";
import { ensureTauriRuntime, invokeCommand } from "./tauriRuntime";

export const HISTORY_EVENTS = {
  indexUpdated: "history:index-updated",
  undoProgress: "history:undo-progress",
  redoProgress: "history:redo-progress"
} as const;

export const deleteHistoryRun = (runId: string) =>
  invokeCommand<void>("delete_history_run", { runId });
export const listHistoryRuns = () => invokeCommand<HistoryIndexEntry[]>("list_history_runs");
export const getHistoryRun = (runId: string) =>
  invokeCommand<HistoryRunDetails | null>("get_history_run", { runId });
export const undoHistoryRun = (runId: string) => invokeCommand<void>("undo_history_run", { runId });
export const redoHistoryRun = (runId: string) => invokeCommand<void>("redo_history_run", { runId });

export interface HistoryListeners {
  onIndexUpdated: (payload: HistoryIndexUpdatedEvent) => void;
  onUndoProgress?: (payload: HistoryUndoProgressEvent) => void;
  onRedoProgress?: (payload: HistoryRedoProgressEvent) => void;
}

export async function attachHistoryListeners(handlers: HistoryListeners): Promise<UnlistenFn[]> {
  ensureTauriRuntime();

  const listeners = await Promise.all([
    listen<HistoryIndexUpdatedEvent>(HISTORY_EVENTS.indexUpdated, (event) =>
      handlers.onIndexUpdated(event.payload)
    )
  ]);
  if (handlers.onUndoProgress) {
    listeners.push(
      await listen<HistoryUndoProgressEvent>(HISTORY_EVENTS.undoProgress, (event) =>
        handlers.onUndoProgress?.(event.payload)
      )
    );
  }
  if (handlers.onRedoProgress) {
    listeners.push(
      await listen<HistoryRedoProgressEvent>(HISTORY_EVENTS.redoProgress, (event) =>
        handlers.onRedoProgress?.(event.payload)
      )
    );
  }

  return listeners;
}
