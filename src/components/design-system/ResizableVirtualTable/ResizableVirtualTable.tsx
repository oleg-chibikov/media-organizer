import { useTable } from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef } from "react";
import { TableHeader } from "./TableHeader";
import { TableBody } from "./TableBody";
import type { ResizableVirtualTableProps } from "./types";
import { TABLE_FEATURES } from "./tableFeatures";
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
  const { columnSizing, setColumnSizing, markColumnAsResized, setColumnWidth } = useColumnSizing({
    rowCount: rows.length,
    columns,
    defaultColumnSizing,
    onAutoSizeColumns
  });

  const table = useTable({
    features: TABLE_FEATURES,
    data: rows,
    columns: columnDefs,
    getRowId: (row) => row.id,
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
    .getAllLeafColumns()
    .map((column) => `${column.getSize()}px`)
    .join(" ");

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
            onResizeTo={(columnId, width) => setColumnWidth(columnId as TColumnKey, width)}
          />
          <TableBody
            rows={rows}
            tableRows={table.getRowModel().rows}
            gridTemplateColumns={gridTemplateColumns}
            rowBehavior={rowBehavior}
            rowVirtualizer={virtualized ? rowVirtualizer : null}
          />
        </div>
      </div>
    </div>
  );
}
