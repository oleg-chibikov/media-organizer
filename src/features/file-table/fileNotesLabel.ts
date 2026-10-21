import type { FileRowState } from "./fileRowState";

const DATE_SOURCE_LABELS: Record<string, string> = {
  SubSecDateTimeOriginal: "Original photo capture time",
  DateTimeOriginal: "Photo capture time",
  MediaCreateDate: "Video creation time",
  TrackCreateDate: "Video track creation time",
  CreateDate: "Embedded media creation time",
  ModifyDate: "Embedded media modified time",
  FileModifyDate: "File modified time (fallback)"
};

export function fileNotesLabel({ metadata, movePlan, moveResult }: FileRowState): string {
  if (moveResult?.error) {
    return moveResult.error;
  }
  if (moveResult?.result_status === "copied") {
    return "Copied to destination; original source file was kept.";
  }
  if (metadata?.metadata_status === "error") {
    return metadata.error ?? "Metadata error";
  }
  if (metadata?.date_kind === "fallback") {
    return "Using FileModifyDate fallback";
  }
  if (!metadata) {
    return "-";
  }
  if (movePlan?.plan_status === "blocked_no_date") {
    return "No usable metadata date";
  }
  if (movePlan?.conflict_status === "suffix_applied") {
    return "Suffix applied for conflict";
  }
  const source = metadata.chosen_date_source;
  return source ? (DATE_SOURCE_LABELS[source] ?? source) : "-";
}
