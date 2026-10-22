import type { ColumnSizingState } from "@tanstack/react-table";
import type { ReactNode } from "react";
import type { TableColumnDef } from "./tableFeatures";

export interface TableColumn<TColumnKey extends string> {
  key: TColumnKey;
  label: string;
}

export interface RowBehavior<TRow> {
  // Mouse only: put a button in a cell for the same action from the keyboard.
  onRowClick?: (row: TRow, rowIndex: number) => void;
  getRowClassName?: (row: TRow, rowIndex: number) => string;
  getRowTooltip?: (row: TRow, rowIndex: number) => string | undefined;
  renderCellWrapper?: (row: TRow, rowIndex: number, columnId: string, cell: ReactNode) => ReactNode;
}

export interface ResizableVirtualTableProps<
  TRow extends { id: string },
  TColumnKey extends string
> extends RowBehavior<TRow> {
  rows: TRow[];
  label: string;
  columns: readonly TableColumn<TColumnKey>[];
  columnDefs: TableColumnDef<TRow>[];
  defaultColumnSizing: ColumnSizingState;
  onAutoSizeColumns: () => ColumnSizingState;
  virtualized?: boolean;
}
