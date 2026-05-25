export type FileKind = "photo" | "video" | "unsupported" | "unknown";
export type ScanStatus = "discovered" | "skipped_processed" | "skipped_unsupported" | "scan_error";

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
export type MetadataStatus = "pending" | "ready" | "no_date" | "error" | "exiftool_restarted";
export type PlaceStatus = "not_available" | "pending" | "resolved" | "failed";

export interface GpsCoordinate {
  latitude: number;
  longitude: number;
}

export interface PlaceComponents {
  landmark: string | null;
  city: string | null;
  region: string | null;
  country: string | null;
}

export interface PlaceCacheEntry {
  version: number;
  osm_place_key: string;
  place: string;
  components: PlaceComponents | null;
  provider: string;
  resolved_at: string;
  last_used_at: string;
  polygon_geojson: string | null;
}

export interface PlaceLookupDiagnosticResult {
  coordinate: GpsCoordinate;
  cached: boolean;
  osm_place_key: string | null;
  provider: string;
  request_url: string;
  http_status: number | null;
  display_place: string | null;
  components: PlaceComponents | null;
  polygon_geojson: string | null;
  raw_response: unknown | null;
  error: string | null;
}

export interface MetadataResult {
  file_id: string;
  chosen_date: string | null;
  chosen_date_source: string | null;
  date_kind: DateKind | null;
  raw_metadata_date: string | null;
  device_name: string | null;
  gps_coordinate: GpsCoordinate | null;
  gps_source: string | null;
  place_bucket_id: string | null;
  place_status: PlaceStatus;
  place: string | null;
  place_error: string | null;
  metadata_status: MetadataStatus;
  error: string | null;
}

export interface MetadataReadRequest {
  file_id: string;
  source_path: string;
  file_kind: FileKind;
}

export interface ProcessingConfig {
  metadata_concurrency: number;
  metadata_queue_capacity: number;
  metadata_timeout_ms: number;
}

export type DestinationRootMode = "custom_root" | "processed_folder";

export interface EffectiveDestinationRoot {
  mode: DestinationRootMode;
  root_path: string;
  custom_root_configured: boolean;
  force_processed_folder: boolean;
  warning: string | null;
}

export interface DestinationSettings {
  version: number;
  root_media_folder: string | null;
  force_processed_folder: boolean;
  updated_at: string;
  warning?: string | null;
  effective_root?: EffectiveDestinationRoot | null;
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

export interface MoveConfirmationSummary {
  total_plans: number;
  ready_operations: number;
  skipped_operations: number;
  blocked_no_date: number;
  blocked_conflict: number;
  plan_errors: number;
  suffix_applied: number;
  fallback_warnings: number;
  photos_ready: number;
  videos_ready: number;
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

export type HistoryRunStatus =
  | "running"
  | "completed"
  | "completed_with_errors"
  | "undone"
  | "partially_undone"
  | "redone"
  | "partial"
  | "corrupt";

export type HistoryOperationKind = "process" | "undo" | "redo";
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

export interface HistoryRun {
  run_id: string;
  history_version: number;
  app_version: string;
  selected_folder: string;
  folder_label: string;
  destination_root_mode?: "custom_root" | "processed_folder" | null;
  destination_root_path?: string | null;
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
