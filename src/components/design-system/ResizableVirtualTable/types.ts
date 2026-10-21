import type { ColumnDef, ColumnSizingState } from "@tanstack/react-table";
import type { ReactNode } from "react";

export interface TableColumn<TColumnKey extends string> {
  key: TColumnKey;
  label: string;
}

export interface RowBehavior<TRow> {
  selectable?: boolean;
  onRowClick?: (row: TRow, rowIndex: number) => void;
  getRowClassName?: (row: TRow, rowIndex: number) => string;
  getRowTooltip?: (row: TRow, rowIndex: number) => string | undefined;
  getRowAriaLabel?: (row: TRow, rowIndex: number) => string | undefined;
  renderCellWrapper?: (row: TRow, rowIndex: number, columnId: string, cell: ReactNode) => ReactNode;
}

export interface ResizableVirtualTableProps<
  TRow extends { id: string },
  TColumnKey extends string
> extends RowBehavior<TRow> {
  rows: TRow[];
  columns: readonly TableColumn<TColumnKey>[];
  columnDefs: ColumnDef<TRow>[];
  defaultColumnSizing: ColumnSizingState;
  onAutoSizeColumns: () => ColumnSizingState;
  virtualized?: boolean;
}
