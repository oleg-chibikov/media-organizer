import type { HistoryOperation } from "./history";

export interface HistoryIndexUpdatedEvent {
  run_id: string | null;
}

// Payload of both `history:undo-progress` and `history:redo-progress`.
export interface HistoryReplayProgressEvent {
  run_id: string;
  total_operations: number;
  completed_operations: number;
  operation: HistoryOperation;
}
