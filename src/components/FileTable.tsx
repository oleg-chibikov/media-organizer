import { type ColumnDef, type ColumnSizingState } from "@tanstack/react-table";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ResizableVirtualTable } from "./design-system/ResizableVirtualTable";
import { revealFileInExplorer, testPlaceLookup } from "../lib/api";
import type { Map as LeafletMap, LayerGroup as LeafletLayerGroup } from "leaflet";
import type {
  FileRecord,
  GpsCoordinate,
  MetadataResult,
  MovePlan,
  MoveResult,
  PlaceLookupDiagnosticResult
} from "../lib/types";

interface FileTableProps {
  rows: FileRecord[];
  metadataByFileId: Record<string, MetadataResult>;
  movePlansByFileId: Record<string, MovePlan>;
  moveResultsByFileId: Record<string, MoveResult>;
  activeMoveFileId: string | null;
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
  { key: "coordinates", label: "Coordinates" },
  { key: "place", label: "Place" },
  { key: "notes", label: "Notes" }
] as const;
const FILE_NAME_COLLATOR = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base"
});

type ColumnKey = (typeof COLUMNS)[number]["key"];

interface CellData {
  value: string;
  title: string;
  className: string;
  fileId?: string;
  coordinate?: GpsCoordinate | null;
}

interface PlaceLookupDiagnosticState {
  loading: boolean;
  result: PlaceLookupDiagnosticResult | null;
  error: string | null;
}

type TableRowData = {
  id: string;
} & Record<ColumnKey, CellData>;

const COLUMN_HORIZONTAL_PADDING = 28;
const MIN_COLUMN_WIDTH = 50;
const MONO_CELL_FONT = "12px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
const BODY_CELL_FONT =
  "14px system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif";
const MONO_COLUMN_KEYS = new Set<ColumnKey>(["source", "destination", "coordinates"]);

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
  activeMoveFileId,
  rootMediaFolder
}: FileTableProps) {
  const [diagnosticsByFileId, setDiagnosticsByFileId] = useState<
    Record<string, PlaceLookupDiagnosticState>
  >({});
  const [activePlaceFileId, setActivePlaceFileId] = useState<string | null>(null);

  const defaultColumnSizing = useMemo<ColumnSizingState>(
    () =>
      COLUMNS.reduce<ColumnSizingState>((accumulator, column) => {
        accumulator[column.key] = COLUMN_HORIZONTAL_PADDING;
        return accumulator;
      }, {}),
    []
  );

  const sortedRows = useMemo<FileRecord[]>(
    () =>
      [...rows].sort((left, right) => {
        const originalNameComparison = FILE_NAME_COLLATOR.compare(
          left.original_name,
          right.original_name
        );
        if (originalNameComparison !== 0) {
          return originalNameComparison;
        }

        const relativePathComparison = FILE_NAME_COLLATOR.compare(
          left.relative_path,
          right.relative_path
        );
        if (relativePathComparison !== 0) {
          return relativePathComparison;
        }

        return FILE_NAME_COLLATOR.compare(left.id, right.id);
      }),
    [rows]
  );

  const tableRows = useMemo<TableRowData[]>(
    () =>
      sortedRows.map((row) => {
        const metadata = metadataByFileId[row.id];
        const movePlan = movePlansByFileId[row.id];
        const moveResult = moveResultsByFileId[row.id];

        return {
          id: row.id,
          source: resolveColumnData(
            "source",
            row,
            metadata,
            movePlan,
            moveResult,
            activeMoveFileId,
            rootMediaFolder
          ),
          destination: resolveColumnData(
            "destination",
            row,
            metadata,
            movePlan,
            moveResult,
            activeMoveFileId,
            rootMediaFolder
          ),
          status: resolveColumnData(
            "status",
            row,
            metadata,
            movePlan,
            moveResult,
            activeMoveFileId,
            rootMediaFolder
          ),
          kind: resolveColumnData(
            "kind",
            row,
            metadata,
            movePlan,
            moveResult,
            activeMoveFileId,
            rootMediaFolder
          ),
          originalName: resolveColumnData(
            "originalName",
            row,
            metadata,
            movePlan,
            moveResult,
            activeMoveFileId,
            rootMediaFolder
          ),
          metadataDate: resolveColumnData(
            "metadataDate",
            row,
            metadata,
            movePlan,
            moveResult,
            activeMoveFileId,
            rootMediaFolder
          ),
          coordinates: resolveColumnData(
            "coordinates",
            row,
            metadata,
            movePlan,
            moveResult,
            activeMoveFileId,
            rootMediaFolder
          ),
          place: resolveColumnData(
            "place",
            row,
            metadata,
            movePlan,
            moveResult,
            activeMoveFileId,
            rootMediaFolder
          ),
          notes: resolveColumnData(
            "notes",
            row,
            metadata,
            movePlan,
            moveResult,
            activeMoveFileId,
            rootMediaFolder
          )
        };
      }),
    [
      sortedRows,
      metadataByFileId,
      movePlansByFileId,
      moveResultsByFileId,
      activeMoveFileId,
      rootMediaFolder
    ]
  );

  const runPlaceDiagnostic = useCallback((fileId: string, coordinate: GpsCoordinate) => {
    setDiagnosticsByFileId((previous) => ({
      ...previous,
      [fileId]: { loading: true, result: previous[fileId]?.result ?? null, error: null }
    }));

    void testPlaceLookup(coordinate)
      .then((result) => {
        setDiagnosticsByFileId((previous) => ({
          ...previous,
          [fileId]: { loading: false, result, error: null }
        }));
      })
      .catch((error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        setDiagnosticsByFileId((previous) => ({
          ...previous,
          [fileId]: { loading: false, result: null, error: message }
        }));
      });
  }, []);

  const handleOpenPlacePane = useCallback(
    (fileId: string) => {
      setActivePlaceFileId(fileId);
      const coordinate = metadataByFileId[fileId]?.gps_coordinate;
      if (coordinate) {
        runPlaceDiagnostic(fileId, coordinate);
      }
    },
    [metadataByFileId, runPlaceDiagnostic]
  );

  const handleClosePlacePane = useCallback(() => {
    setActivePlaceFileId(null);
  }, []);

  const tableColumns = useMemo<ColumnDef<TableRowData>[]>(
    () =>
      COLUMNS.map((column) => ({
        id: column.key,
        header: column.label,
        accessorKey: column.key,
        minSize: HEADER_MIN_WIDTHS[column.key],
        cell: (context) => {
          const data = context.getValue() as CellData;
          if (column.key === "place") {
            const diagnostic = data.fileId ? diagnosticsByFileId[data.fileId] : undefined;
            return (
              <PlaceCell
                data={data}
                diagnostic={diagnostic}
                onOpenPlacePane={(fileId) => handleOpenPlacePane(fileId)}
              />
            );
          }

          return (
            <div className={data.className} title={data.title}>
              {data.value}
            </div>
          );
        }
      })),
    [diagnosticsByFileId, handleOpenPlacePane]
  );

  const calculateAutoSizedColumns = useCallback(() => {
    const desiredWidths: Record<ColumnKey, number> = {
      source: HEADER_MIN_WIDTHS.source,
      destination: HEADER_MIN_WIDTHS.destination,
      status: HEADER_MIN_WIDTHS.status,
      kind: HEADER_MIN_WIDTHS.kind,
      originalName: HEADER_MIN_WIDTHS.originalName,
      metadataDate: HEADER_MIN_WIDTHS.metadataDate,
      coordinates: HEADER_MIN_WIDTHS.coordinates,
      place: HEADER_MIN_WIDTHS.place,
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
  }, [sortedRows.length, tableRows]);

  const activePlaceMetadata = activePlaceFileId ? metadataByFileId[activePlaceFileId] : undefined;
  const activePlaceRow = activePlaceFileId
    ? sortedRows.find((row) => row.id === activePlaceFileId)
    : undefined;
  const activePlaceDiagnostic = activePlaceFileId
    ? diagnosticsByFileId[activePlaceFileId]
    : undefined;

  return (
    <div className="relative h-full">
      <ResizableVirtualTable
        rows={tableRows}
        columns={COLUMNS}
        columnDefs={tableColumns}
        defaultColumnSizing={defaultColumnSizing}
        onAutoSizeColumns={calculateAutoSizedColumns}
        getRowClassName={(_, rowIndex) => {
          const sourceRecord = sortedRows[rowIndex];
          const metadata = metadataByFileId[sourceRecord.id];
          const moveResult = moveResultsByFileId[sourceRecord.id];
          const isSkippedUnsupported = sourceRecord.scan_status === "skipped_unsupported";
          const isFallbackWarning = metadata?.date_kind === "fallback";
          const isMoved = moveResult?.result_status === "moved";
          const isCopied = moveResult?.result_status === "copied";
          const isActiveMove = activeMoveFileId === sourceRecord.id && !moveResult;
          const isMoveError =
            moveResult?.result_status === "skipped" || moveResult?.result_status === "failed";
          return `${isMoved ? "bg-emerald-950/30 text-emerald-200" : ""} ${isCopied ? "bg-sky-950/30 text-sky-200" : ""} ${isActiveMove ? "bg-cyan-950/30 text-cyan-100" : ""} ${isMoveError || isSkippedUnsupported ? "bg-rose-950/30 text-rose-200" : ""} ${isFallbackWarning ? "bg-amber-950/20" : ""}`;
        }}
        getRowTooltip={(row) => row.source.value}
        renderCellWrapper={(_, rowIndex, columnId, cell) => {
          const sourceRecord = sortedRows[rowIndex];
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

      {activePlaceFileId ? (
        <PlaceDetailsPane
          row={activePlaceRow}
          metadata={activePlaceMetadata}
          diagnostic={activePlaceDiagnostic}
          onClose={handleClosePlacePane}
          onRetry={() => {
            const coordinate = activePlaceMetadata?.gps_coordinate;
            if (activePlaceFileId && coordinate) {
              runPlaceDiagnostic(activePlaceFileId, coordinate);
            }
          }}
        />
      ) : null}
    </div>
  );
}

function PlaceCell({
  data,
  diagnostic,
  onOpenPlacePane
}: {
  data: CellData;
  diagnostic: PlaceLookupDiagnosticState | undefined;
  onOpenPlacePane: (fileId: string) => void;
}) {
  return (
    <button
      type="button"
      className="flex w-full min-w-0 items-center gap-2 px-3 py-2 text-left hover:bg-slate-900/60"
      title={data.title}
      onClick={(event) => {
        event.stopPropagation();
        if (data.fileId) {
          onOpenPlacePane(data.fileId);
        }
      }}
      disabled={!data.fileId}
    >
      <span className="truncate">{data.value}</span>
      {diagnostic?.loading ? (
        <span className="shrink-0 rounded border border-slate-700 px-1.5 py-0.5 text-[11px] text-cyan-200">
          Checking
        </span>
      ) : null}
    </button>
  );
}

function PlaceDetailsPane({
  row,
  metadata,
  diagnostic,
  onClose,
  onRetry
}: {
  row: FileRecord | undefined;
  metadata: MetadataResult | undefined;
  diagnostic: PlaceLookupDiagnosticState | undefined;
  onClose: () => void;
  onRetry: () => void;
}) {
  const coordinate = diagnostic?.result?.coordinate ?? metadata?.gps_coordinate ?? null;
  const placeName = diagnostic?.result?.display_place ?? metadata?.place ?? "Not available";
  const requestUsed = diagnostic?.result?.request_url ?? "No request has been made yet.";
  const responseText = formatJsonForDisplay(diagnostic?.result?.raw_response);

  return (
    <aside className="absolute inset-y-0 right-0 z-20 w-full max-w-[40rem] border-l border-slate-800 bg-slate-950/95 shadow-2xl backdrop-blur">
      <div className="flex h-full flex-col">
        <div className="flex items-start justify-between gap-3 border-b border-slate-800 px-4 py-3">
          <div className="min-w-0">
            <h2 className="truncate text-sm font-semibold uppercase tracking-wide text-slate-200">
              Place Details
            </h2>
            <p className="truncate text-xs text-slate-400" title={row?.relative_path ?? "-"}>
              {row?.relative_path ?? "-"}
            </p>
          </div>
          <button
            type="button"
            className="rounded border border-slate-700 px-2 py-1 text-xs text-slate-200 hover:border-cyan-500 hover:text-cyan-100"
            onClick={onClose}
          >
            Close
          </button>
        </div>

        <div className="min-h-0 flex-1 space-y-4 overflow-auto p-4">
          <section className="space-y-2 rounded border border-slate-800 bg-slate-900/60 p-3">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-slate-300">
              Summary
            </h3>
            <p className="text-sm text-slate-200">{placeName || "Not available"}</p>
            <p className="text-xs text-slate-400">
              Coordinates: {formatGpsCoordinate(coordinate) ?? "None"}
            </p>
            <p className="text-xs text-slate-400">
              Cached: {diagnostic?.result ? (diagnostic.result.cached ? "Yes" : "No") : "Unknown"}
            </p>
            <p className="text-xs text-slate-400">
              HTTP status: {diagnostic?.result?.http_status ?? "none"}
            </p>
            {diagnostic?.error ? (
              <p className="text-xs text-rose-300">Diagnostic error: {diagnostic.error}</p>
            ) : null}
            {diagnostic?.result?.error ? (
              <p className="text-xs text-rose-300">Lookup error: {diagnostic.result.error}</p>
            ) : null}
            {coordinate ? (
              <button
                type="button"
                className="rounded border border-slate-700 px-2 py-1 text-xs text-slate-200 hover:border-cyan-500 hover:text-cyan-100 disabled:cursor-wait disabled:opacity-60"
                disabled={diagnostic?.loading}
                onClick={onRetry}
              >
                {diagnostic?.loading ? "Refreshing..." : "Refresh lookup"}
              </button>
            ) : null}
          </section>

          <section className="space-y-2 rounded border border-slate-800 bg-slate-900/60 p-3">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-slate-300">
              Request
            </h3>
            <pre className="overflow-auto rounded border border-slate-800 bg-slate-950 p-2 text-xs text-slate-300">
              {requestUsed}
            </pre>
          </section>

          <section className="space-y-2 rounded border border-slate-800 bg-slate-900/60 p-3">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-slate-300">
              Response
            </h3>
            <pre className="max-h-64 overflow-auto rounded border border-slate-800 bg-slate-950 p-2 text-xs text-slate-300">
              {responseText}
            </pre>
          </section>

          <section className="space-y-2 rounded border border-slate-800 bg-slate-900/60 p-3">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-slate-300">Map</h3>
            <PlaceMap
              coordinate={coordinate}
              polygonGeoJson={diagnostic?.result?.polygon_geojson ?? null}
            />
          </section>
        </div>
      </div>
    </aside>
  );
}

function PlaceMap({
  coordinate,
  polygonGeoJson
}: {
  coordinate: GpsCoordinate | null;
  polygonGeoJson: string | null;
}) {
  const mapContainerRef = useRef<HTMLDivElement | null>(null);
  const mapRef = useRef<LeafletMap | null>(null);
  const layersRef = useRef<LeafletLayerGroup | null>(null);
  const [mapError, setMapError] = useState<string | null>(null);

  useEffect(() => {
    let disposed = false;
    let cleanup: (() => void) | undefined;

    void import("leaflet")
      .then((leaflet) => {
        if (disposed || !mapContainerRef.current) {
          return;
        }

        const map = leaflet.map(mapContainerRef.current, {
          zoomControl: true,
          attributionControl: true
        });
        leaflet
          .tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
            maxZoom: 19,
            attribution: "&copy; OpenStreetMap contributors"
          })
          .addTo(map);
        const layers = leaflet.layerGroup().addTo(map);

        mapRef.current = map;
        layersRef.current = layers;
        cleanup = () => {
          layers.clearLayers();
          map.remove();
          mapRef.current = null;
          layersRef.current = null;
        };
      })
      .catch((error: unknown) => {
        setMapError(`Map failed to load: ${String(error)}`);
      });

    return () => {
      disposed = true;
      cleanup?.();
    };
  }, []);

  useEffect(() => {
    if (!mapRef.current || !layersRef.current) {
      return;
    }

    const map = mapRef.current;
    const layers = layersRef.current;
    layers.clearLayers();

    void import("leaflet")
      .then((leaflet) => {
        let hasBounds = false;
        const bounds = leaflet.latLngBounds([]);

        if (polygonGeoJson) {
          try {
            const parsed = JSON.parse(polygonGeoJson);
            const polygonLayer = leaflet.geoJSON(parsed as never, {
              style: {
                color: "#22d3ee",
                weight: 2,
                fillColor: "#0891b2",
                fillOpacity: 0.2
              }
            });
            polygonLayer.addTo(layers);
            const polygonBounds = polygonLayer.getBounds();
            if (polygonBounds.isValid()) {
              bounds.extend(polygonBounds);
              hasBounds = true;
            }
          } catch (error) {
            setMapError(`Invalid polygon GeoJSON: ${String(error)}`);
          }
        }

        if (coordinate) {
          const marker = leaflet.circleMarker([coordinate.latitude, coordinate.longitude], {
            radius: 6,
            color: "#f97316",
            weight: 2,
            fillColor: "#fb923c",
            fillOpacity: 0.9
          });
          marker.bindTooltip("Coordinate");
          marker.addTo(layers);
          bounds.extend([coordinate.latitude, coordinate.longitude]);
          hasBounds = true;
        }

        if (hasBounds && bounds.isValid()) {
          map.fitBounds(bounds.pad(0.2));
        } else {
          map.setView([0, 0], 2);
        }
        window.requestAnimationFrame(() => {
          map.invalidateSize();
        });
      })
      .catch((error: unknown) => {
        setMapError(`Map failed to render: ${String(error)}`);
      });
  }, [coordinate, polygonGeoJson]);

  if (!coordinate && !polygonGeoJson) {
    return <p className="text-xs text-slate-400">No coordinates or polygon available for map.</p>;
  }

  return (
    <div className="space-y-2">
      {mapError ? <p className="text-xs text-rose-300">{mapError}</p> : null}
      <div ref={mapContainerRef} className="h-64 w-full rounded border border-slate-800" />
    </div>
  );
}

function resolveColumnData(
  columnKey: ColumnKey,
  row: FileRecord,
  metadata: MetadataResult | undefined,
  movePlan: MovePlan | undefined,
  moveResult: MoveResult | undefined,
  activeMoveFileId: string | null,
  rootMediaFolder: string | null
): CellData {
  const metadataDateLabel = metadata
    ? (metadata.chosen_date ?? metadata.raw_metadata_date ?? "No metadata date")
    : "pending";
  const destinationLabel =
    movePlan?.destination_path || (metadata?.metadata_status === "ready" ? "planning..." : "-");
  const relativeDestinationLabel = resolveRelativePath(destinationLabel, rootMediaFolder);
  const statusLabel = resolveStatusLabel(row, metadata, movePlan, moveResult, activeMoveFileId);
  const coordinateCellData = resolveCoordinateCellData(metadata);
  const placeCellData = resolvePlaceCellData(metadata);
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
    case "coordinates":
      return {
        value: coordinateCellData.value,
        title: coordinateCellData.title,
        className: "truncate px-3 py-2 font-mono text-xs text-slate-300",
        fileId: row.id,
        coordinate: metadata?.gps_coordinate ?? null
      };
    case "place":
      return {
        value: placeCellData.value,
        title: placeCellData.title,
        className: "truncate px-3 py-2",
        fileId: row.id,
        coordinate: metadata?.gps_coordinate ?? null
      };
    case "notes":
      return { value: notesLabel, title: notesLabel, className: "truncate px-3 py-2" };
  }
}

function formatJsonForDisplay(value: unknown): string {
  if (value === null || value === undefined) {
    return "No response payload.";
  }

  if (typeof value === "string") {
    return value;
  }

  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
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
  moveResult: MoveResult | undefined,
  activeMoveFileId: string | null
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

  if (activeMoveFileId === row.id) {
    return "Moving";
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

function resolvePlaceCellData(metadata: MetadataResult | undefined): CellData {
  if (!metadata) {
    return { value: "-", title: "-", className: "" };
  }

  const coordinateLabel = formatGpsCoordinate(metadata.gps_coordinate);

  switch (metadata.place_status) {
    case "pending":
      return {
        value: "Pending lookup",
        title: coordinateLabel
          ? `Pending lookup (Coordinates: ${coordinateLabel})`
          : "Pending lookup",
        className: ""
      };
    case "resolved": {
      const placeLabel = metadata.place?.trim() || "Resolved";
      return {
        value: placeLabel,
        title: coordinateLabel ? `${placeLabel} (Coordinates: ${coordinateLabel})` : placeLabel,
        className: ""
      };
    }
    case "failed": {
      const baseTitle = metadata.place_error?.trim()
        ? `Lookup failed: ${metadata.place_error}`
        : "Lookup failed";
      const title = coordinateLabel ? `${baseTitle} (Coordinates: ${coordinateLabel})` : baseTitle;
      return { value: "Lookup failed", title, className: "" };
    }
    case "not_available":
    default: {
      const reason = resolvePlaceUnavailableReason(metadata);
      const title = coordinateLabel
        ? `Not available: ${reason} (Coordinates: ${coordinateLabel})`
        : `Not available: ${reason}`;
      return { value: "Not available", title, className: "" };
    }
  }
}

function resolveCoordinateCellData(metadata: MetadataResult | undefined): CellData {
  if (!metadata) {
    return { value: "-", title: "-", className: "" };
  }

  if (!metadata.gps_coordinate) {
    return { value: "-", title: "No GPS coordinates in metadata", className: "" };
  }

  const coordinateLabel = formatGpsCoordinate(metadata.gps_coordinate);
  if (!coordinateLabel) {
    return { value: "-", title: "GPS coordinates are unavailable", className: "" };
  }

  const invalidReason = resolveInvalidCoordinateReason(metadata.gps_coordinate);
  const title = invalidReason ? `${coordinateLabel} (${invalidReason})` : coordinateLabel;
  return { value: coordinateLabel, title, className: "" };
}

function resolvePlaceUnavailableReason(metadata: MetadataResult): string {
  if (!metadata.gps_coordinate) {
    return "No GPS coordinates in metadata";
  }

  const invalidReason = resolveInvalidCoordinateReason(metadata.gps_coordinate);
  if (invalidReason) {
    return invalidReason;
  }

  return "GPS coordinates are present but place lookup did not produce a place.";
}

function resolveInvalidCoordinateReason(coordinate: GpsCoordinate): string | null {
  if (!Number.isFinite(coordinate.latitude) || !Number.isFinite(coordinate.longitude)) {
    return "GPS coordinates are not finite numbers";
  }

  if (coordinate.latitude < -90 || coordinate.latitude > 90) {
    return "GPS latitude is out of range (-90 to 90)";
  }

  if (coordinate.longitude < -180 || coordinate.longitude > 180) {
    return "GPS longitude is out of range (-180 to 180)";
  }

  return null;
}

function formatGpsCoordinate(coordinate: GpsCoordinate | null): string | null {
  if (!coordinate) {
    return null;
  }

  return `${formatGpsValue(coordinate.latitude)}, ${formatGpsValue(coordinate.longitude)}`;
}

function formatGpsValue(value: number): string {
  return value.toFixed(6).replace(/\.?0+$/, "");
}

function resolveDateSourceLabel(source: string | null): string {
  if (!source) {
    return "-";
  }

  return DATE_SOURCE_LABELS[source] ?? source;
}
