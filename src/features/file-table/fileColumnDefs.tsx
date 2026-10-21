import type { ColumnDef } from "@tanstack/react-table";
import { FILE_TABLE_COLUMNS, type FileCell, type FileTableRow } from "./fileTableColumns";
import { HEADER_MIN_WIDTHS } from "./measureColumnWidths";

export const FILE_COLUMN_DEFS: ColumnDef<FileTableRow>[] = FILE_TABLE_COLUMNS.map((column) => ({
  id: column.key,
  header: column.label,
  accessorKey: column.key,
  minSize: HEADER_MIN_WIDTHS[column.key],
  cell: (context) => {
    const data = context.getValue() as FileCell;
    return (
      <div className={data.className} title={data.title}>
        {data.value}
      </div>
    );
  }
}));
