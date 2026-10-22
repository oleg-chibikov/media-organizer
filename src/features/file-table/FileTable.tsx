import type { FileIdMap } from "../../lib/fileIdMap";
import { useCallback, useMemo, useState } from "react";
import { ResizableVirtualTable } from "../../components/design-system/ResizableVirtualTable";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";
import { FileFilterBar } from "./FileFilterBar";
import { createFileRowCache } from "./fileRowCache";
import { fileFilterRange, type FileFilter } from "./fileFilter";
import { FILE_COLUMN_DEFS } from "./fileColumnDefs";
import { fileRowClassName } from "./fileRowClassName";
import { FILE_TABLE_COLUMNS } from "./fileTableColumns";
import { DEFAULT_COLUMN_SIZING, measureAutoSizedColumns } from "./measureColumnWidths";
import { PlaceMapDialog } from "../place-map";
import { CellActions } from "./CellActions";
import { sortFileRecords } from "./sortFileRecords";
import { sortProblemsFirst } from "./sortProblemsFirst";
import { usePlaceMapTarget } from "./usePlaceMapTarget";

interface FileTableProps {
  rows: FileRecord[];
  metadataByFileId: FileIdMap<MetadataResult>;
  movePlansByFileId: FileIdMap<MovePlan>;
  planErrorsByFileId: Record<string, string>;
  moveResultsByFileId: FileIdMap<MoveResult>;
  activeMoveFileId: string | null;
  destinationRoot: string | null;
  // Place labels by file id, empty until detection answers.
  filePlaces: Record<string, string>;
  locatingPlaces: boolean;
}

const EMPTY_FILTER_MESSAGES: Record<FileFilter, string> = {
  all: "No files yet.",
  review: "No files need review.",
  error: "No errors."
};

export function FileTable({
  rows,
  metadataByFileId,
  movePlansByFileId,
  planErrorsByFileId,
  moveResultsByFileId,
  activeMoveFileId,
  destinationRoot,
  filePlaces,
  locatingPlaces
}: FileTableProps) {
  const [filter, setFilter] = useState<FileFilter>("all");
  const { mapTarget, openMap, closeMap } = usePlaceMapTarget(filePlaces);
  const sortedRows = useMemo(() => sortFileRecords(rows), [rows]);
  const [rowCache] = useState(createFileRowCache);

  const sorted = useMemo(
    () =>
      sortProblemsFirst(
        rowCache.states(sortedRows, {
          metadataByFileId,
          movePlansByFileId,
          moveResultsByFileId,
          planErrorsByFileId
        }),
        rowCache.categoryOf
      ),
    [
      rowCache,
      sortedRows,
      metadataByFileId,
      movePlansByFileId,
      moveResultsByFileId,
      planErrorsByFileId
    ]
  );

  const tableRows = useMemo(
    () =>
      rowCache.rows(sorted.states, {
        activeMoveFileId,
        destinationRoot,
        filePlaces,
        locatingPlaces
      }),
    [rowCache, sorted, activeMoveFileId, destinationRoot, filePlaces, locatingPlaces]
  );

  // Columns fit every row, so switching the filter doesn't change their widths.
  const autoSizeColumns = useCallback(() => measureAutoSizedColumns(tableRows), [tableRows]);

  const [start, end] = fileFilterRange(filter, sorted);
  const visibleStates = sorted.states.slice(start, end);
  const visibleRows = useMemo(() => tableRows.slice(start, end), [tableRows, start, end]);

  return (
    <div className="flex h-full flex-col">
      {rows.length > 0 ? (
        <FileFilterBar
          value={filter}
          total={rows.length}
          reviewCount={sorted.reviewCount}
          errorCount={sorted.errorCount}
          onChange={setFilter}
        />
      ) : null}
      <div className="min-h-0 flex-1">
        {visibleRows.length === 0 && rows.length > 0 ? (
          <p className="px-3 py-4 text-sm text-slate-400">{EMPTY_FILTER_MESSAGES[filter]}</p>
        ) : (
          <ResizableVirtualTable
            rows={visibleRows}
            label="Files"
            columns={FILE_TABLE_COLUMNS}
            columnDefs={FILE_COLUMN_DEFS}
            defaultColumnSizing={DEFAULT_COLUMN_SIZING}
            onAutoSizeColumns={autoSizeColumns}
            getRowClassName={(_, rowIndex) =>
              fileRowClassName(visibleStates[rowIndex], activeMoveFileId)
            }
            getRowTooltip={(row) => row.source.value}
            renderCellWrapper={(_, rowIndex, columnId, cell) => (
              <CellActions state={visibleStates[rowIndex]} columnId={columnId} onOpenMap={openMap}>
                {cell}
              </CellActions>
            )}
          />
        )}
      </div>
      {mapTarget ? <PlaceMapDialog target={mapTarget} onClose={closeMap} /> : null}
    </div>
  );
}
