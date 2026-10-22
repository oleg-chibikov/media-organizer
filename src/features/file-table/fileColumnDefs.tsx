import type { TableColumnDef } from "../../components/design-system/ResizableVirtualTable";
import { LoadingText } from "../../components/design-system/LoadingText";
import { PlaceIcon } from "../../components/design-system/PlaceIcon";
import { FILE_TABLE_COLUMNS, type FileCell, type FileTableRow } from "./fileTableColumns";
import { HEADER_MIN_WIDTHS } from "./measureColumnWidths";

export const FILE_COLUMN_DEFS: TableColumnDef<FileTableRow>[] = FILE_TABLE_COLUMNS.map(
  (column) => ({
    id: column.key,
    header: column.label,
    accessorKey: column.key,
    minSize: HEADER_MIN_WIDTHS[column.key],
    cell: (context) => {
      const data = context.getValue() as FileCell;
      return (
        <div className={data.className} title={data.title}>
          {data.icon ? <PlaceIcon kind={data.icon} /> : null}
          {data.loading ? <LoadingText>{data.value}</LoadingText> : data.value}
        </div>
      );
    }
  })
);
