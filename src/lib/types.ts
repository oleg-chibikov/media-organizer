export type FileKind = "photo" | "video" | "unsupported" | "unknown";
export type ScanStatus =
  | "discovered"
  | "skipped_processed"
  | "skipped_unsupported"
  | "scan_error";

export interface FileRecord {
  id: string;
  source_path: string;
  relative_path: string;
  extension: string;
  original_name: string;
  kind: FileKind;
  scan_status: ScanStatus;
}

export type DateKind = "timezone_aware" | "timezone_unknown" | "fallback";
export type MetadataStatus =
  | "pending"
  | "ready"
  | "no_date"
  | "error"
  | "exiftool_restarted";

export interface MetadataResult {
  file_id: string;
  chosen_date: string | null;
  chosen_date_source: string | null;
  date_kind: DateKind | null;
  raw_metadata_date: string | null;
  metadata_status: MetadataStatus;
  error: string | null;
}

export type ConflictStatus = "none" | "suffix_applied" | "blocked";
export type PlanStatus = "ready" | "blocked_no_date" | "blocked_conflict" | "error";

export interface MovePlan {
  file_id: string;
  original_name: string;
  destination_dir: string;
  destination_filename: string;
  destination_path: string;
  conflict_status: ConflictStatus;
  plan_status: PlanStatus;
}

export type MoveStrategy = "rename" | "copy_remove" | "not_attempted";
export type MoveResultStatus = "moved" | "skipped" | "failed";

export interface MoveResult {
  file_id: string;
  attempted_source_path: string;
  attempted_destination_path: string;
  move_strategy: MoveStrategy;
  result_status: MoveResultStatus;
  error: string | null;
}

export interface PlanFileEvent {
  plan: MovePlan;
}

export interface PlanErrorEvent {
  file_id: string;
  error: string;
}

export interface PlanFinishedEvent {
  planned: number;
  errors: number;
}

export interface ScanStartedEvent {
  root: string;
}

export interface ScanFileEvent {
  record: FileRecord;
}

export interface ScanErrorEvent {
  path: string;
  error: string;
}

export interface ScanFinishedEvent {
  discovered: number;
  skipped: number;
  errors: number;
  cancelled: boolean;
}
