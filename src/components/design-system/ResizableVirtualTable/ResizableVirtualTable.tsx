import { getCoreRowModel, useReactTable } from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef } from "react";
import { TableHeader } from "./TableHeader";
import { TableRow } from "./TableRow";
import type { ResizableVirtualTableProps } from "./types";
import { useColumnSizing } from "./useColumnSizing";

export function ResizableVirtualTable<TRow extends { id: string }, TColumnKey extends string>({
  rows,
  label,
  columns,
  columnDefs,
  defaultColumnSizing,
  onAutoSizeColumns,
  virtualized = true,
  ...rowBehavior
}: ResizableVirtualTableProps<TRow, TColumnKey>) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const { columnSizing, setColumnSizing, markColumnAsResized } = useColumnSizing({
    rowCount: rows.length,
    columns,
    defaultColumnSizing,
    onAutoSizeColumns
  });

  const table = useReactTable({
    data: rows,
    columns: columnDefs,
    getRowId: (row) => row.id,
    getCoreRowModel: getCoreRowModel(),
    columnResizeMode: "onChange",
    enableColumnResizing: true,
    state: { columnSizing },
    onColumnSizingChange: setColumnSizing
  });

  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 38,
    overscan: 20,
    enabled: virtualized
  });

  const gridTemplateColumns = table
    .getVisibleLeafColumns()
    .map((column) => `${column.getSize()}px`)
    .join(" ");
  const tableRows = table.getRowModel().rows;

  return (
    <div className="h-full overflow-hidden">
      <div ref={parentRef} className="h-full overflow-auto [scrollbar-gutter:stable]">
        <div
          aria-label={label}
          aria-rowcount={rows.length + 1}
          className="min-h-full"
          role="table"
          style={{ width: `max(100%, ${table.getTotalSize()}px)` }}
        >
          <TableHeader
            table={table}
            gridTemplateColumns={gridTemplateColumns}
            onResizeStart={(columnId) => markColumnAsResized(columnId as TColumnKey)}
          />
          {virtualized ? (
            <div
              className="relative"
              role="rowgroup"
              style={{ height: `${rowVirtualizer.getTotalSize()}px` }}
            >
              {rowVirtualizer.getVirtualItems().map((virtualRow) => (
                <TableRow
                  key={tableRows[virtualRow.index].id}
                  {...rowBehavior}
                  row={tableRows[virtualRow.index]}
                  sourceRow={rows[virtualRow.index]}
                  rowIndex={virtualRow.index}
                  layoutClassName="absolute left-0 top-0 grid w-full"
                  style={{
                    gridTemplateColumns,
                    transform: `translateY(${virtualRow.start}px)`,
                    height: `${virtualRow.size}px`
                  }}
                />
              ))}
            </div>
          ) : (
            <div className="min-h-full" role="rowgroup">
              {tableRows.map((row, rowIndex) => (
                <TableRow
                  key={row.id}
                  {...rowBehavior}
                  row={row}
                  sourceRow={rows[rowIndex]}
                  rowIndex={rowIndex}
                  layoutClassName="grid w-full"
                  style={{ gridTemplateColumns }}
                />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
