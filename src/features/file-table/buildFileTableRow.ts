import type { FileTableRow } from "./fileTableColumns";
import { fileNotesLabel } from "./fileNotesLabel";
import type { FileRowState } from "./fileRowState";
import { fileStatusLabel } from "./fileStatusLabel";
import { relativeDestinationPath } from "./relativeDestinationPath";

const TEXT_CELL = "truncate px-3 py-2";

function textCell(value: string) {
  return { value, title: value, className: TEXT_CELL };
}

export function buildFileTableRow(
  state: FileRowState,
  activeMoveFileId: string | null,
  destinationRoot: string | null
): FileTableRow {
  const { record, metadata, movePlan } = state;
  const metadataDate = metadata
    ? (metadata.chosen_date ?? metadata.raw_metadata_date ?? "No metadata date")
    : "pending";
  const destination =
    movePlan?.destination_path || (metadata?.metadata_status === "ready" ? "planning..." : "-");

  return {
    id: record.id,
    source: {
      value: record.relative_path,
      title: record.relative_path,
      className: "truncate px-3 py-2 font-mono text-xs"
    },
    destination: {
      value: relativeDestinationPath(destination, destinationRoot),
      title: destination,
      className: "truncate px-3 py-2 font-mono text-xs text-slate-300"
    },
    status: textCell(fileStatusLabel(state, activeMoveFileId)),
    date: textCell(metadataDate),
    kind: textCell(record.kind),
    notes: textCell(fileNotesLabel(state))
  };
}
