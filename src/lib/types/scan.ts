export type FileKind = "photo" | "video" | "unsupported" | "unknown";
type ScanStatus = "discovered" | "skipped_processed" | "skipped_unsupported" | "scan_error";

export interface FileRecord {
  id: string;
  source_path: string;
  relative_path: string;
  extension: string;
  original_name: string;
  kind: FileKind;
  scan_status: ScanStatus;
}

export interface ScanStartedEvent {
  root: string;
}

export interface ScanFilesEvent {
  records: FileRecord[];
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
