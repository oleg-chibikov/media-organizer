import type { FilePlace } from "../../lib/types";
import type { FileCell, FileTableRow } from "./fileTableColumns";
import { fileNotesLabel } from "./fileNotesLabel";
import type { FileRowState } from "./fileRowState";
import { fileStatus } from "./fileStatus";
import { relativeDestinationPath } from "./relativeDestinationPath";

const TEXT_CELL = "truncate px-3 py-2";
const PATH_CELL = "truncate px-3 py-2 font-mono text-xs";

function textCell(value: string): FileCell {
  return { value, title: value, className: TEXT_CELL };
}

// LoadingText adds the dots on screen, so the tooltip gets them here.
function loadingCell(value: string): FileCell {
  return { value, title: `${value}...`, className: TEXT_CELL, loading: true };
}

function placeCell(value: string | undefined, atHome: boolean): FileCell {
  return value ? { ...textCell(value), icon: atHome ? "home" : "away" } : textCell("");
}

// The folder the plan puts the file in, or what the plan still waits for.
function destinationCell(
  { record, metadata, movePlan, planError }: FileRowState,
  destinationRoot: string | null
): FileCell {
  if (movePlan?.destination_path) {
    const path = movePlan.destination_path;
    const value = relativeDestinationPath(path, destinationRoot);
    return { value, title: path, className: `${PATH_CELL} text-slate-300` };
  }
  if (record.scan_status !== "discovered" || planError || movePlan) {
    return textCell("-");
  }
  if (!metadata) {
    return loadingCell("Reading");
  }
  return metadata.metadata_status === "ready" ? loadingCell("Planning") : textCell("-");
}

export function buildFileTableRow(
  state: FileRowState,
  activeMoveFileId: string | null,
  destinationRoot: string | null,
  place: FilePlace | undefined,
  locatingPlace: boolean
): FileTableRow {
  const { record, metadata } = state;
  const date =
    record.scan_status !== "discovered"
      ? textCell("-")
      : metadata
        ? textCell(metadata.chosen_date ?? metadata.raw_metadata_date ?? "No date")
        : loadingCell("Reading");
  const status = fileStatus(state, activeMoveFileId);
  // Only photos and videos get a place, so other rows stay blank while locating.
  // A place from a preview shows at once.
  const locating = !place && locatingPlace && (record.kind === "photo" || record.kind === "video");

  return {
    id: record.id,
    source: { value: record.relative_path, title: record.relative_path, className: PATH_CELL },
    destination: destinationCell(state, destinationRoot),
    status: status.busy ? loadingCell(status.label) : textCell(status.label),
    date,
    country: placeCell(place?.country, place?.home !== "away"),
    place: locating ? loadingCell("Locating") : placeCell(place?.place, place?.home === "city"),
    kind: textCell(record.kind),
    notes: textCell(fileNotesLabel(state))
  };
}
