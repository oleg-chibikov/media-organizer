import type {
  HistoryIndexEntry,
  HistoryIndexUpdatedEvent,
  HistoryReplayProgressEvent,
  HistoryRunDetails
} from "../types";
import { listenAll } from "./listenAll";
import { invokeCommand } from "./tauriRuntime";

export const deleteHistoryRun = (runId: string) =>
  invokeCommand<void>("delete_history_run", { runId });
export const listHistoryRuns = () => invokeCommand<HistoryIndexEntry[]>("list_history_runs");
export const getHistoryRun = (runId: string) =>
  invokeCommand<HistoryRunDetails | null>("get_history_run", { runId });
export const undoHistoryRun = (runId: string) => invokeCommand<void>("undo_history_run", { runId });
export const redoHistoryRun = (runId: string) => invokeCommand<void>("redo_history_run", { runId });

export interface HistoryListeners {
  onIndexUpdated: (payload: HistoryIndexUpdatedEvent) => void;
  onUndoProgress?: (payload: HistoryReplayProgressEvent) => void;
  onRedoProgress?: (payload: HistoryReplayProgressEvent) => void;
}

const HISTORY_EVENTS = {
  onIndexUpdated: "history:index-updated",
  onUndoProgress: "history:undo-progress",
  onRedoProgress: "history:redo-progress"
} as const;

export const attachHistoryListeners = (handlers: HistoryListeners) =>
  listenAll(HISTORY_EVENTS, handlers);
