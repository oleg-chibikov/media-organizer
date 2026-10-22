import {
  columnResizingFeature,
  columnSizingFeature,
  tableFeatures,
  type ColumnDef,
  type Row,
  type Table
} from "@tanstack/react-table";

export const TABLE_FEATURES = tableFeatures({ columnSizingFeature, columnResizingFeature });

type TableFeatureSet = typeof TABLE_FEATURES;

export type TableColumnDef<TRow extends object> = ColumnDef<TableFeatureSet, TRow>;
export type TableRowModel<TRow extends object> = Row<TableFeatureSet, TRow>;
export type TableInstance<TRow extends object> = Table<TableFeatureSet, TRow>;
