import type { FilePlace } from "../../lib/types";
import type { FileCell, FileTableRow } from "./fileTableColumns";
import { fileNotesLabel } from "./fileNotesLabel";
import type { FileRowState } from "./fileRowState";
import { fileStatusLabel } from "./fileStatusLabel";
import { relativeDestinationPath } from "./relativeDestinationPath";

const TEXT_CELL = "truncate px-3 py-2";
const PATH_CELL = "truncate px-3 py-2 font-mono text-xs";

function textCell(value: string): FileCell {
  return { value, title: value, className: TEXT_CELL };
}

function loadingCell(value: string): FileCell {
  return { ...textCell(value), loading: true };
}

// The folder the plan puts the file in, or what the plan still waits for.
function destinationCell(
  { metadata, movePlan }: FileRowState,
  destinationRoot: string | null
): FileCell {
  if (movePlan?.destination_path) {
    const path = movePlan.destination_path;
    const value = relativeDestinationPath(path, destinationRoot);
    return { value, title: path, className: `${PATH_CELL} text-slate-300` };
  }
  return metadata?.metadata_status === "ready" ? loadingCell("Planning...") : textCell("-");
}

export function buildFileTableRow(
  state: FileRowState,
  activeMoveFileId: string | null,
  destinationRoot: string | null,
  place: FilePlace | undefined,
  locatingPlace: boolean
): FileTableRow {
  const { record, metadata } = state;
  const metadataDate =
    record.scan_status !== "discovered"
      ? "-"
      : metadata
        ? (metadata.chosen_date ?? metadata.raw_metadata_date ?? "No date")
        : "Reading...";
  // Only photos and videos get a place, so other rows stay blank while locating.
  // A place from a preview shows at once.
  const locating = !place && locatingPlace && (record.kind === "photo" || record.kind === "video");

  return {
    id: record.id,
    source: { value: record.relative_path, title: record.relative_path, className: PATH_CELL },
    destination: destinationCell(state, destinationRoot),
    status: textCell(fileStatusLabel(state, activeMoveFileId)),
    date: textCell(metadataDate),
    country: textCell(place?.country ?? ""),
    place: locating ? loadingCell("Locating...") : textCell(place?.place ?? ""),
    kind: textCell(record.kind),
    notes: textCell(fileNotesLabel(state))
  };
}
