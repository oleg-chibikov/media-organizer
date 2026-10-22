import type { MoveStrategy } from "./move";

export type HistoryRunStatus =
  | "running"
  | "completed"
  | "completed_with_errors"
  | "undone"
  | "partially_undone"
  | "redone"
  | "partial"
  | "corrupt";

type HistoryOperationKind = "process" | "undo" | "redo";
export type HistoryOperationStatus = "succeeded" | "skipped" | "blocked" | "failed";

export interface HistoryIndexEntry {
  run_id: string;
  folder_label: string;
  selected_folder: string;
  destination_root_path?: string | null;
  started_at: string;
  completed_at: string | null;
  run_status: HistoryRunStatus;
  files_migrated?: number;
  undo_available: boolean;
  redo_available: boolean;
  history_file_path: string;
}

interface HistoryRun {
  run_id: string;
  history_version: number;
  app_version: string;
  selected_folder: string;
  folder_label: string;
  destination_root_mode?: "custom_root" | "processed_folder" | null;
  destination_root_path?: string | null;
  /** Where place folders went. Differs from the destination root when renaming in place. */
  place_root_path?: string | null;
  started_at: string;
  completed_at: string | null;
  run_status: HistoryRunStatus;
  history_file_path: string;
}

export interface HistoryOperation {
  operation_id: string;
  run_id: string;
  file_id: string;
  operation_kind: HistoryOperationKind;
  source_path: string;
  destination_path: string;
  expected_source_size: number | null;
  expected_destination_size: number | null;
  metadata_date: string | null;
  original_name: string;
  move_strategy: MoveStrategy;
  status: HistoryOperationStatus;
  error: string | null;
  timestamp: string;
}

export interface HistoryRunDetails {
  run: HistoryRun;
  operations: HistoryOperation[];
  unsafe_for_undo_redo: boolean;
  recovery_error: string | null;
}
