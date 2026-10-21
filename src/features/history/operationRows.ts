import type { ColumnSizingState } from "@tanstack/react-table";
import type { HistoryRunDetails } from "../../lib/types";
import { historyDestinationRoot, toRelativeHistoryPath } from "./historyPaths";
import { formatOperationStatus } from "./historyStatusLabels";

export const OPERATION_COLUMNS = [
  { key: "source", label: "Source" },
  { key: "destination", label: "Destination" },
  { key: "status", label: "Status" }
] as const;

type OperationColumnKey = (typeof OPERATION_COLUMNS)[number]["key"];
export type OperationTableRow = { id: string; error: string } & Record<OperationColumnKey, string>;

export const OPERATION_DEFAULT_SIZING: ColumnSizingState = {
  source: 300,
  destination: 300,
  status: 120
};

export function buildOperationRows(
  details: HistoryRunDetails,
  pathDisplay: "absolute" | "relative"
): OperationTableRow[] {
  const sourceRoot = details.run.selected_folder;
  const destinationRoot = historyDestinationRoot(details.run);
  const display = (path: string, root: string) =>
    pathDisplay === "relative" ? toRelativeHistoryPath(path, root) : path;

  return details.operations.map((operation) => ({
    id: operation.operation_id,
    status: formatOperationStatus(operation.status),
    source: display(operation.source_path, sourceRoot),
    destination: display(operation.destination_path, destinationRoot),
    error: operation.error ?? ""
  }));
}
