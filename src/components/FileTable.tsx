import { type ColumnDef, type ColumnSizingState } from "@tanstack/react-table";
import { useCallback, useMemo } from "react";
import { ResizableVirtualTable } from "./design-system/ResizableVirtualTable";
import { revealFileInExplorer } from "../lib/api";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../lib/types";

interface FileTableProps {
  rows: FileRecord[];
  metadataByFileId: Record<string, MetadataResult>;
  movePlansByFileId: Record<string, MovePlan>;
  moveResultsByFileId: Record<string, MoveResult>;
  rootMediaFolder: string | null;
}

const STATUS_LABELS: Record<FileRecord["scan_status"], string> = {
  discovered: "Discovered",
  skipped_processed: "Skipped Processed",
  skipped_unsupported: "Skipped Unsupported",
  scan_error: "Scan Error"
};

const DATE_SOURCE_LABELS: Record<string, string> = {
  SubSecDateTimeOriginal: "Original photo capture time",
  DateTimeOriginal: "Photo capture time",
  MediaCreateDate: "Video creation time",
  TrackCreateDate: "Video track creation time",
  CreateDate: "Embedded media creation time",
  ModifyDate: "Embedded media modified time",
  FileModifyDate: "File modified time (fallback)"
};

const COLUMNS = [
  { key: "source", label: "Source File" },
  { key: "destination", label: "Destination" },
  { key: "status", label: "Status" },
  { key: "kind", label: "Kind" },
  { key: "originalName", label: "Original Name" },
  { key: "metadataDate", label: "Metadata Date" },
  { key: "notes", label: "Notes" }
] as const;

type ColumnKey = (typeof COLUMNS)[number]["key"];

interface CellData {
  value: string;
  title: string;
  className: string;
}

type TableRowData = {
  id: string;
} & Record<ColumnKey, CellData>;

const COLUMN_HORIZONTAL_PADDING = 28;
const MIN_COLUMN_WIDTH = 50;
const MONO_CELL_FONT = "12px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
const BODY_CELL_FONT =
  "14px system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif";
const MONO_COLUMN_KEYS = new Set<ColumnKey>(["source", "destination"]);

const HEADER_MIN_WIDTHS: Record<ColumnKey, number> = (() => {
  if (typeof document === "undefined") {
    return COLUMNS.reduce<Record<ColumnKey, number>>(
      (accumulator, column) => {
        accumulator[column.key] = MIN_COLUMN_WIDTH;
        return accumulator;
      },
      {} as Record<ColumnKey, number>
    );
  }

  const canvas = document.createElement("canvas");
  const context = canvas.getContext("2d");

  if (!context) {
    return COLUMNS.reduce<Record<ColumnKey, number>>(
      (accumulator, column) => {
        accumulator[column.key] = MIN_COLUMN_WIDTH;
        return accumulator;
      },
      {} as Record<ColumnKey, number>
    );
  }

  context.font = BODY_CELL_FONT;

  return COLUMNS.reduce<Record<ColumnKey, number>>(
    (accumulator, column) => {
      const headerWidth = Math.ceil(context.measureText(column.label).width);
      accumulator[column.key] = Math.max(MIN_COLUMN_WIDTH, headerWidth + COLUMN_HORIZONTAL_PADDING);
      return accumulator;
    },
    {} as Record<ColumnKey, number>
  );
})();

export function FileTable({
  rows,
  metadataByFileId,
  movePlansByFileId,
  moveResultsByFileId,
  rootMediaFolder
}: FileTableProps) {
  const defaultColumnSizing = useMemo<ColumnSizingState>(
    () =>
      COLUMNS.reduce<ColumnSizingState>((accumulator, column) => {
        accumulator[column.key] = COLUMN_HORIZONTAL_PADDING;
        return accumulator;
      }, {}),
    []
  );

  const tableRows = useMemo<TableRowData[]>(
    () =>
      rows.map((row) => {
        const metadata = metadataByFileId[row.id];
        const movePlan = movePlansByFileId[row.id];
        const moveResult = moveResultsByFileId[row.id];

        return {
          id: row.id,
          source: resolveColumnData("source", row, metadata, movePlan, moveResult, rootMediaFolder),
          destination: resolveColumnData(
            "destination",
            row,
            metadata,
            movePlan,
            moveResult,
            rootMediaFolder
          ),
          status: resolveColumnData("status", row, metadata, movePlan, moveResult, rootMediaFolder),
          kind: resolveColumnData("kind", row, metadata, movePlan, moveResult, rootMediaFolder),
          originalName: resolveColumnData(
            "originalName",
            row,
            metadata,
            movePlan,
            moveResult,
            rootMediaFolder
          ),
          metadataDate: resolveColumnData(
            "metadataDate",
            row,
            metadata,
            movePlan,
            moveResult,
            rootMediaFolder
          ),
          notes: resolveColumnData("notes", row, metadata, movePlan, moveResult, rootMediaFolder)
        };
      }),
    [rows, metadataByFileId, movePlansByFileId, moveResultsByFileId, rootMediaFolder]
  );

  const tableColumns = useMemo<ColumnDef<TableRowData>[]>(
    () =>
      COLUMNS.map((column) => ({
        id: column.key,
        header: column.label,
        accessorKey: column.key,
        minSize: HEADER_MIN_WIDTHS[column.key],
        cell: (context) => {
          const data = context.getValue() as CellData;
          return (
            <div className={data.className} title={data.title}>
              {data.value}
            </div>
          );
        }
      })),
    []
  );

  const calculateAutoSizedColumns = useCallback(() => {
    const desiredWidths: Record<ColumnKey, number> = {
      source: HEADER_MIN_WIDTHS.source,
      destination: HEADER_MIN_WIDTHS.destination,
      status: HEADER_MIN_WIDTHS.status,
      kind: HEADER_MIN_WIDTHS.kind,
      originalName: HEADER_MIN_WIDTHS.originalName,
      metadataDate: HEADER_MIN_WIDTHS.metadataDate,
      notes: HEADER_MIN_WIDTHS.notes
    };

    if (rows.length === 0) {
      return desiredWidths;
    }

    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
      return desiredWidths;
    }

    const measureWidth = (value: string, font: string) => {
      context.font = font;
      return Math.ceil(context.measureText(value).width);
    };

    COLUMNS.forEach((column) => {
      const cellMaxWidth = tableRows.reduce((maxWidth, row) => {
        const font = MONO_COLUMN_KEYS.has(column.key) ? MONO_CELL_FONT : BODY_CELL_FONT;
        return Math.max(maxWidth, measureWidth(row[column.key].value, font));
      }, 0);

      desiredWidths[column.key] = Math.max(
        HEADER_MIN_WIDTHS[column.key],
        cellMaxWidth + COLUMN_HORIZONTAL_PADDING
      );
    });

    return desiredWidths;
  }, [rows.length, tableRows]);

  return (
    <ResizableVirtualTable
      rows={tableRows}
      columns={COLUMNS}
      columnDefs={tableColumns}
      defaultColumnSizing={defaultColumnSizing}
      onAutoSizeColumns={calculateAutoSizedColumns}
      getRowClassName={(_, rowIndex) => {
        const sourceRecord = rows[rowIndex];
        const metadata = metadataByFileId[sourceRecord.id];
        const moveResult = moveResultsByFileId[sourceRecord.id];
        const isSkippedUnsupported = sourceRecord.scan_status === "skipped_unsupported";
        const isFallbackWarning = metadata?.date_kind === "fallback";
        const isMoved = moveResult?.result_status === "moved";
        const isCopied = moveResult?.result_status === "copied";
        const isMoveError =
          moveResult?.result_status === "skipped" || moveResult?.result_status === "failed";
        return `${isMoved ? "bg-emerald-950/30 text-emerald-200" : ""} ${isCopied ? "bg-sky-950/30 text-sky-200" : ""} ${isMoveError || isSkippedUnsupported ? "bg-rose-950/30 text-rose-200" : ""} ${isFallbackWarning ? "bg-amber-950/20" : ""}`;
      }}
      getRowTooltip={(row) => row.source.value}
      renderCellWrapper={(_, rowIndex, columnId, cell) => {
        const sourceRecord = rows[rowIndex];
        const movePlan = movePlansByFileId[sourceRecord.id];
        const moveResult = moveResultsByFileId[sourceRecord.id];
        const isSourceCell = columnId === "source";
        const isDestinationCell = columnId === "destination";
        const destinationPath =
          moveResult?.attempted_destination_path ?? movePlan?.destination_path;
        const revealPath = isSourceCell
          ? sourceRecord.source_path
          : isDestinationCell
            ? destinationPath
            : null;

        return (
          <div
            className={isSourceCell || isDestinationCell ? "cursor-pointer" : undefined}
            onDoubleClick={() => {
              if (!revealPath) {
                return;
              }

              void revealFileInExplorer(revealPath).catch((error: unknown) => {
                console.error("Failed to reveal file in explorer", error);
              });
            }}
          >
            {cell}
          </div>
        );
      }}
    />
  );
}

function resolveColumnData(
  columnKey: ColumnKey,
  row: FileRecord,
  metadata: MetadataResult | undefined,
  movePlan: MovePlan | undefined,
  moveResult: MoveResult | undefined,
  rootMediaFolder: string | null
): { value: string; title: string; className: string } {
  const metadataDateLabel = metadata
    ? (metadata.chosen_date ?? metadata.raw_metadata_date ?? "No metadata date")
    : "pending";
  const destinationLabel =
    movePlan?.destination_path || (metadata?.metadata_status === "ready" ? "planning..." : "-");
  const relativeDestinationLabel = resolveRelativePath(destinationLabel, rootMediaFolder);
  const statusLabel = resolveStatusLabel(row, metadata, movePlan, moveResult);
  const notesLabel = resolveNotesLabel(metadata, movePlan, moveResult);

  switch (columnKey) {
    case "source":
      return {
        value: row.relative_path,
        title: row.relative_path,
        className: "truncate px-3 py-2 font-mono text-xs"
      };
    case "destination":
      return {
        value: relativeDestinationLabel,
        title: destinationLabel,
        className: "truncate px-3 py-2 font-mono text-xs text-slate-300"
      };
    case "status":
      return { value: statusLabel, title: statusLabel, className: "truncate px-3 py-2" };
    case "kind":
      return { value: row.kind, title: row.kind, className: "truncate px-3 py-2" };
    case "originalName":
      return {
        value: row.original_name,
        title: row.original_name,
        className: "truncate px-3 py-2"
      };
    case "metadataDate":
      return {
        value: metadataDateLabel,
        title: metadataDateLabel,
        className: "truncate px-3 py-2"
      };
    case "notes":
      return { value: notesLabel, title: notesLabel, className: "truncate px-3 py-2" };
  }
}

function resolveRelativePath(pathValue: string, basePath: string | null): string {
  if (!basePath || pathValue === "-" || pathValue === "planning...") {
    return pathValue;
  }

  const normalizedBase = normalizePath(basePath);
  const normalizedPath = normalizePath(pathValue);
  if (!normalizedPath.toLowerCase().startsWith(`${normalizedBase.toLowerCase()}/`)) {
    return pathValue;
  }

  return normalizedPath.slice(normalizedBase.length + 1);
}

function normalizePath(pathValue: string): string {
  return pathValue.replace(/\\/g, "/").replace(/\/+$/, "");
}

function resolveStatusLabel(
  row: FileRecord,
  metadata: MetadataResult | undefined,
  movePlan: MovePlan | undefined,
  moveResult: MoveResult | undefined
): string {
  if (moveResult) {
    switch (moveResult.result_status) {
      case "moved":
        return "Moved";
      case "copied":
        return "Copied (source kept)";
      case "skipped":
        return "Skipped";
      case "failed":
        return "Failed";
      default:
        return "Processed";
    }
  }

  if (row.scan_status !== "discovered") {
    return STATUS_LABELS[row.scan_status];
  }

  if (metadata?.metadata_status === "error") {
    return "Metadata Error";
  }
  if (metadata?.date_kind === "fallback") {
    return "Warning Fallback";
  }

  if (!movePlan) {
    return metadata?.metadata_status === "ready" ? "Planning" : "Pending Metadata";
  }

  switch (movePlan.plan_status) {
    case "ready":
      return movePlan.conflict_status === "suffix_applied" ? "Ready (Suffix)" : "Ready";
    case "blocked_no_date":
      return "Blocked No Date";
    case "blocked_conflict":
      return "Blocked Conflict";
    case "error":
      return "Plan Error";
    default:
      return "Pending Plan";
  }
}

function resolveNotesLabel(
  metadata: MetadataResult | undefined,
  movePlan: MovePlan | undefined,
  moveResult: MoveResult | undefined
): string {
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

  return resolveDateSourceLabel(metadata.chosen_date_source);
}

function resolveDateSourceLabel(source: string | null): string {
  if (!source) {
    return "-";
  }

  return DATE_SOURCE_LABELS[source] ?? source;
}
