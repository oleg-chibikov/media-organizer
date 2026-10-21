import { useCallback, useMemo } from "react";
import { ResizableVirtualTable } from "../../components/design-system/ResizableVirtualTable";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";
import { buildFileTableRow } from "./buildFileTableRow";
import { FILE_COLUMN_DEFS } from "./fileColumnDefs";
import { fileRowClassName } from "./fileRowClassName";
import type { FileRowState } from "./fileRowState";
import { FILE_TABLE_COLUMNS } from "./fileTableColumns";
import { DEFAULT_COLUMN_SIZING, measureAutoSizedColumns } from "./measureColumnWidths";
import { RevealOnDoubleClick } from "./RevealOnDoubleClick";
import { sortFileRecords } from "./sortFileRecords";

interface FileTableProps {
  rows: FileRecord[];
  metadataByFileId: Record<string, MetadataResult>;
  movePlansByFileId: Record<string, MovePlan>;
  moveResultsByFileId: Record<string, MoveResult>;
  activeMoveFileId: string | null;
  rootMediaFolder: string | null;
}

export function FileTable({
  rows,
  metadataByFileId,
  movePlansByFileId,
  moveResultsByFileId,
  activeMoveFileId,
  rootMediaFolder
}: FileTableProps) {
  const sortedRows = useMemo(() => sortFileRecords(rows), [rows]);

  const rowStates = useMemo<FileRowState[]>(
    () =>
      sortedRows.map((record) => ({
        record,
        metadata: metadataByFileId[record.id],
        movePlan: movePlansByFileId[record.id],
        moveResult: moveResultsByFileId[record.id]
      })),
    [sortedRows, metadataByFileId, movePlansByFileId, moveResultsByFileId]
  );

  const tableRows = useMemo(
    () => rowStates.map((state) => buildFileTableRow(state, activeMoveFileId, rootMediaFolder)),
    [rowStates, activeMoveFileId, rootMediaFolder]
  );

  const autoSizeColumns = useCallback(() => measureAutoSizedColumns(tableRows), [tableRows]);

  return (
    <ResizableVirtualTable
      rows={tableRows}
      columns={FILE_TABLE_COLUMNS}
      columnDefs={FILE_COLUMN_DEFS}
      defaultColumnSizing={DEFAULT_COLUMN_SIZING}
      onAutoSizeColumns={autoSizeColumns}
      getRowClassName={(_, rowIndex) => fileRowClassName(rowStates[rowIndex], activeMoveFileId)}
      getRowTooltip={(row) => row.source.value}
      renderCellWrapper={(_, rowIndex, columnId, cell) => (
        <RevealOnDoubleClick state={rowStates[rowIndex]} columnId={columnId}>
          {cell}
        </RevealOnDoubleClick>
      )}
    />
  );
}
