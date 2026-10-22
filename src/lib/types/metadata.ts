import type { FileKind } from "./scan";

type DateKind = "timezone_aware" | "timezone_unknown" | "fallback";
type MetadataStatus = "pending" | "ready" | "no_date" | "error" | "exiftool_restarted";

/** Signed decimal degrees. South and west are negative. */
export interface GpsPoint {
  latitude: number;
  longitude: number;
}

export interface MetadataResult {
  file_id: string;
  chosen_date: string | null;
  chosen_date_source: string | null;
  date_kind: DateKind | null;
  raw_metadata_date: string | null;
  device_name: string | null;
  gps: GpsPoint | null;
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
