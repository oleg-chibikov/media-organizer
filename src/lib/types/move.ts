export type MoveStrategy = "rename" | "copy_keep_source" | "not_attempted";
export type MoveResultStatus = "moved" | "copied" | "skipped" | "failed";

export interface MoveResult {
  file_id: string;
  attempted_source_path: string;
  attempted_destination_path: string;
  move_strategy: MoveStrategy;
  result_status: MoveResultStatus;
  error: string | null;
}

export interface MoveFileEvent {
  result: MoveResult;
}

export type MoveProgressPhase = "starting" | "preparing" | "moving" | "finalizing";

export interface MoveProgressEvent {
  phase: MoveProgressPhase;
  file_id: string | null;
  message: string;
  completed: number;
  total: number;
}

export interface MoveFinishedEvent {
  moved: number;
  copied: number;
  skipped: number;
  failed: number;
  results: MoveResult[];
}
