import type { HistoryOperation } from "./history";

export interface HistoryIndexUpdatedEvent {
  run_id: string | null;
}

export interface HistoryUndoProgressEvent {
  run_id: string;
  total_operations: number;
  completed_operations: number;
  operation: HistoryOperation;
}

export interface HistoryRedoProgressEvent {
  run_id: string;
  total_operations: number;
  completed_operations: number;
  operation: HistoryOperation;
}
