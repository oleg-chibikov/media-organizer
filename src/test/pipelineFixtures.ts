import type { FileRecord, MetadataResult, MovePlan } from "../lib/types";

export const RECORD: FileRecord = {
  id: "f1",
  source_path: "C:\\in\\a.jpg",
  relative_path: "a.jpg",
  extension: "jpg",
  original_name: "a.jpg",
  kind: "photo",
  scan_status: "discovered"
};

export const METADATA: MetadataResult = {
  file_id: "f1",
  chosen_date: "2026-05-24T00:00:00Z",
  chosen_date_source: "DateTimeOriginal",
  date_kind: "timezone_aware",
  raw_metadata_date: null,
  device_name: null,
  gps: null,
  metadata_status: "ready",
  error: null
};

export const PLAN: MovePlan = {
  file_id: "f1",
  original_name: "a.jpg",
  destination_dir: "C:\\in\\Processed\\2026",
  destination_filename: "a.jpg",
  destination_path: "C:\\in\\Processed\\2026\\a.jpg",
  conflict_status: "none",
  plan_status: "ready",
  placement: null
};
