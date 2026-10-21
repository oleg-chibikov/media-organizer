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

// The header is row 1, so data rows start at aria-rowindex 2.
const FIRST_DATA_ROW_INDEX = 2;

export function TableRow<TRow>({
  row,
  sourceRow,
  rowIndex,
  layoutClassName,
  style,
  onRowClick,
  getRowClassName,
  getRowTooltip,
  renderCellWrapper
}: TableRowProps<TRow>) {
  const rowClassName = getRowClassName?.(sourceRow, rowIndex) ?? "";

  return (
    // A row click is a mouse shortcut; keyboard users get a real control inside the row.
    <div
      aria-rowindex={rowIndex + FIRST_DATA_ROW_INDEX}
      className={`${layoutClassName} border-t border-slate-800 text-sm ${onRowClick ? "cursor-pointer" : ""} ${rowClassName}`}
      role="row"
      title={getRowTooltip?.(sourceRow, rowIndex)}
      onClick={onRowClick ? () => onRowClick(sourceRow, rowIndex) : undefined}
      style={style}
    >
      {row.getVisibleCells().map((cell) => {
        const renderedCell = flexRender(cell.column.columnDef.cell, cell.getContext());
        const wrappedCell = renderCellWrapper?.(sourceRow, rowIndex, cell.column.id, renderedCell);
        return (
          <div key={cell.id} role="cell">
            {wrappedCell ?? renderedCell}
          </div>
        );
      })}
    </div>
  );
}
