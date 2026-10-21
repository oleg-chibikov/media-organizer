import type { FileRowState } from "./fileRowState";

const DATE_SOURCE_LABELS: Record<string, string> = {
  SubSecDateTimeOriginal: "Photo capture time",
  DateTimeOriginal: "Photo capture time",
  MediaCreateDate: "Video recording time",
  TrackCreateDate: "Video recording time",
  CreateDate: "Creation time saved in the file",
  ModifyDate: "Edit time saved in the file",
  FileModifyDate: "Date the file last changed"
};

export function fileNotesLabel({
  metadata,
  movePlan,
  moveResult,
  planError
}: FileRowState): string {
  if (moveResult?.error) {
    return moveResult.error;
  }
  if (moveResult?.result_status === "copied") {
    return "Copied. The original file stays in place.";
  }
  if (metadata?.metadata_status === "error") {
    return metadata.error ?? "Can't read the date from this file";
  }
  if (planError && !movePlan) {
    return planError;
  }
  if (metadata?.date_kind === "fallback") {
    return "No capture date in the file. Using the date the file last changed.";
  }
  if (!metadata) {
    return "-";
  }
  if (movePlan?.plan_status === "blocked_no_date") {
    return "No capture date found. This file stays in place.";
  }
  if (movePlan?.plan_status === "already_named") {
    return "This file already has its new name. It stays as it is.";
  }
  if (movePlan?.conflict_status === "suffix_applied") {
    return "A file with this name is already there. A number is added.";
  }
  const source = metadata.chosen_date_source;
  return source ? (DATE_SOURCE_LABELS[source] ?? source) : "-";
}
