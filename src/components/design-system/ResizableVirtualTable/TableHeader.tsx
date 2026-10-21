import { flexRender, type Table } from "@tanstack/react-table";
import type { MouseEvent, TouchEvent } from "react";
import { Button } from "../Button";

interface TableHeaderProps<TRow> {
  table: Table<TRow>;
  gridTemplateColumns: string;
  onResizeStart: (columnId: string) => void;
}

export function TableHeader<TRow>({
  table,
  gridTemplateColumns,
  onResizeStart
}: TableHeaderProps<TRow>) {
  return (
    <div
      aria-rowindex={1}
      role="row"
      className="sticky top-0 z-10 grid w-full border-b border-slate-800 bg-slate-950 text-left text-xs uppercase tracking-wide text-slate-400"
      style={{ gridTemplateColumns }}
    >
      {table.getHeaderGroups().map((headerGroup) =>
        headerGroup.headers.map((header) => {
          const startResize = (event: MouseEvent | TouchEvent) => {
            onResizeStart(header.column.id);
            header.getResizeHandler()(event);
          };
          return (
            <div key={header.id} className="relative px-3 py-2" role="columnheader">
              <span className="truncate">
                {header.isPlaceholder
                  ? null
                  : flexRender(header.column.columnDef.header, header.getContext())}
              </span>
              <Button
                variant="secondary"
                aria-label={`Resize ${String(header.column.columnDef.header)} column`}
                className="absolute inset-y-0 right-0 w-2 cursor-col-resize touch-none bg-transparent hover:bg-slate-700/60 focus:bg-slate-700/60 focus:outline-none"
                onMouseDown={startResize}
                onTouchStart={startResize}
              >
                <span className="sr-only">Resize</span>
              </Button>
            </div>
          );
        })
      )}
    </div>
  );
}
