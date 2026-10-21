import { flexRender, type Row } from "@tanstack/react-table";
import type { CSSProperties } from "react";
import type { RowBehavior } from "./types";

interface TableRowProps<TRow> extends RowBehavior<TRow> {
  row: Row<TRow>;
  sourceRow: TRow;
  rowIndex: number;
  layoutClassName: string;
  style: CSSProperties;
}

export function TableRow<TRow>({
  row,
  sourceRow,
  rowIndex,
  layoutClassName,
  style,
  selectable = false,
  onRowClick,
  getRowClassName,
  getRowTooltip,
  getRowAriaLabel,
  renderCellWrapper
}: TableRowProps<TRow>) {
  const rowClassName = getRowClassName?.(sourceRow, rowIndex) ?? "";

  return (
    <div
      aria-label={selectable ? getRowAriaLabel?.(sourceRow, rowIndex) : undefined}
      className={`${layoutClassName} border-t border-slate-800 text-sm ${selectable ? "cursor-pointer" : ""} ${rowClassName}`}
      role={selectable ? "button" : undefined}
      tabIndex={selectable ? 0 : undefined}
      title={getRowTooltip?.(sourceRow, rowIndex)}
      onClick={() => {
        if (selectable) {
          onRowClick?.(sourceRow, rowIndex);
        }
      }}
      onKeyDown={(event) => {
        if (!selectable || (event.key !== "Enter" && event.key !== " ")) {
          return;
        }
        event.preventDefault();
        onRowClick?.(sourceRow, rowIndex);
      }}
      style={style}
    >
      {row.getVisibleCells().map((cell) => {
        const renderedCell = flexRender(cell.column.columnDef.cell, cell.getContext());
        const wrappedCell = renderCellWrapper?.(sourceRow, rowIndex, cell.column.id, renderedCell);
        return <div key={cell.id}>{wrappedCell ?? renderedCell}</div>;
      })}
    </div>
  );
}
