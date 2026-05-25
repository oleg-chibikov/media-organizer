import { useVirtualizer } from "@tanstack/react-virtual";
import {
  flexRender,
  getCoreRowModel,
  type ColumnDef,
  type ColumnSizingState,
  useReactTable
} from "@tanstack/react-table";
import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { Button } from "./Button";

interface ResizableVirtualTableProps<TRow extends { id: string }, TColumnKey extends string> {
  rows: TRow[];
  columns: readonly { key: TColumnKey; label: string }[];
  columnDefs: ColumnDef<TRow>[];
  defaultColumnSizing: ColumnSizingState;
  onAutoSizeColumns: () => ColumnSizingState;
  selectable?: boolean;
  virtualized?: boolean;
  onRowClick?: (row: TRow, rowIndex: number) => void;
  getRowClassName?: (row: TRow, rowIndex: number) => string;
  getRowTooltip?: (row: TRow, rowIndex: number) => string | undefined;
  renderCellWrapper?: (row: TRow, rowIndex: number, columnId: string, cell: ReactNode) => ReactNode;
}

export function ResizableVirtualTable<TRow extends { id: string }, TColumnKey extends string>({
  rows,
  columns,
  columnDefs,
  defaultColumnSizing,
  onAutoSizeColumns,
  selectable = false,
  virtualized = true,
  onRowClick,
  getRowClassName,
  getRowTooltip,
  renderCellWrapper
}: ResizableVirtualTableProps<TRow, TColumnKey>) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const manuallyResizedColumnsRef = useRef<Set<TColumnKey>>(new Set());
  const [columnSizing, setColumnSizing] = useState<ColumnSizingState>(defaultColumnSizing);

  const table = useReactTable({
    data: rows,
    columns: columnDefs,
    getCoreRowModel: getCoreRowModel(),
    columnResizeMode: "onChange",
    enableColumnResizing: true,
    state: { columnSizing },
    onColumnSizingChange: setColumnSizing
  });

  const gridTemplateColumns = table
    .getVisibleLeafColumns()
    .map((column) => `${column.getSize()}px`)
    .join(" ");

  const tableMinWidth = table.getTotalSize();
  const tableWidth = `max(100%, ${tableMinWidth}px)`;

  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 38,
    overscan: 20,
    enabled: virtualized
  });

  const markColumnAsResized = useCallback((columnKey: TColumnKey) => {
    manuallyResizedColumnsRef.current.add(columnKey);
    document.body.classList.add("cursor-col-resize", "select-none");
  }, []);

  useEffect(() => {
    if (rows.length === 0) {
      return;
    }

    const desiredWidths = onAutoSizeColumns();
    setColumnSizing((previous) => {
      let changed = false;
      const next: ColumnSizingState = { ...previous };

      for (const column of columns) {
        if (manuallyResizedColumnsRef.current.has(column.key)) {
          continue;
        }

        const desiredWidth = desiredWidths[column.key];
        if (next[column.key] !== desiredWidth) {
          next[column.key] = desiredWidth;
          changed = true;
        }
      }

      return changed ? next : previous;
    });
  }, [columns, onAutoSizeColumns, rows.length]);

  useEffect(() => {
    const stopResizing = () => {
      document.body.classList.remove("cursor-col-resize", "select-none");
    };

    window.addEventListener("pointerup", stopResizing);
    return () => {
      window.removeEventListener("pointerup", stopResizing);
      document.body.classList.remove("cursor-col-resize", "select-none");
    };
  }, []);

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-x-auto">
        <div className="flex h-full w-full flex-col" style={{ width: tableWidth }}>
          <div
            className="grid w-full border-b border-slate-800 bg-slate-950 text-left text-xs uppercase tracking-wide text-slate-400"
            style={{ gridTemplateColumns }}
          >
            {table.getHeaderGroups().map((headerGroup) =>
              headerGroup.headers.map((header) => (
                <div key={header.id} className="relative px-3 py-2">
                  <span className="truncate">
                    {header.isPlaceholder
                      ? null
                      : flexRender(header.column.columnDef.header, header.getContext())}
                  </span>
                  <Button
                    variant="secondary"
                    aria-label={`Resize ${String(header.column.columnDef.header)} column`}
                    className="absolute inset-y-0 right-0 w-2 cursor-col-resize touch-none bg-transparent hover:bg-slate-700/60 focus:bg-slate-700/60 focus:outline-none"
                    onMouseDown={(event) => {
                      markColumnAsResized(header.column.id as TColumnKey);
                      header.getResizeHandler()(event);
                    }}
                    onTouchStart={(event) => {
                      markColumnAsResized(header.column.id as TColumnKey);
                      header.getResizeHandler()(event);
                    }}
                  >
                    <span className="sr-only">Resize</span>
                  </Button>
                </div>
              ))
            )}
          </div>

          <div
            ref={parentRef}
            className="min-h-0 flex-1 overflow-y-auto overflow-x-hidden [scrollbar-gutter:stable]"
          >
            {virtualized ? (
              <div className="relative" style={{ height: `${rowVirtualizer.getTotalSize()}px` }}>
                {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                  const row = table.getRowModel().rows[virtualRow.index];
                  const sourceRow = rows[virtualRow.index];
                  const rowClassName = getRowClassName?.(sourceRow, virtualRow.index) ?? "";
                  const rowTooltip = getRowTooltip?.(sourceRow, virtualRow.index);

                  return (
                    <div
                      key={row.id}
                      className={`absolute left-0 top-0 grid w-full border-t border-slate-800 text-sm ${selectable ? "cursor-pointer" : ""} ${rowClassName}`}
                      title={rowTooltip}
                      onClick={() => {
                        if (selectable) {
                          onRowClick?.(sourceRow, virtualRow.index);
                        }
                      }}
                      style={{
                        gridTemplateColumns,
                        transform: `translateY(${virtualRow.start}px)`,
                        height: `${virtualRow.size}px`
                      }}
                    >
                      {row.getVisibleCells().map((cell) => {
                        const renderedCell = flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext()
                        );
                        const wrappedCell = renderCellWrapper?.(
                          sourceRow,
                          virtualRow.index,
                          cell.column.id,
                          renderedCell
                        );
                        return <div key={cell.id}>{wrappedCell ?? renderedCell}</div>;
                      })}
                    </div>
                  );
                })}
              </div>
            ) : (
              <div className="min-h-full">
                {table.getRowModel().rows.map((row, rowIndex) => {
                  const sourceRow = rows[rowIndex];
                  const rowClassName = getRowClassName?.(sourceRow, rowIndex) ?? "";
                  const rowTooltip = getRowTooltip?.(sourceRow, rowIndex);
                  return (
                    <div
                      key={row.id}
                      className={`grid w-full border-t border-slate-800 text-sm ${selectable ? "cursor-pointer" : ""} ${rowClassName}`}
                      title={rowTooltip}
                      onClick={() => {
                        if (selectable) {
                          onRowClick?.(sourceRow, rowIndex);
                        }
                      }}
                      style={{ gridTemplateColumns }}
                    >
                      {row.getVisibleCells().map((cell) => {
                        const renderedCell = flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext()
                        );
                        const wrappedCell = renderCellWrapper?.(
                          sourceRow,
                          rowIndex,
                          cell.column.id,
                          renderedCell
                        );
                        return <div key={cell.id}>{wrappedCell ?? renderedCell}</div>;
                      })}
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
