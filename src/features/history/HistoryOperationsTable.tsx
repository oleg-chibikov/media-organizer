import {
  ResizableVirtualTable,
  type TableColumnDef
} from "../../components/design-system/ResizableVirtualTable";
import {
  OPERATION_COLUMNS,
  OPERATION_DEFAULT_SIZING,
  type OperationTableRow
} from "./operationRows";

const keepDefaultSizing = () => OPERATION_DEFAULT_SIZING;

const OPERATION_COLUMN_DEFS: TableColumnDef<OperationTableRow>[] = OPERATION_COLUMNS.map(
  (column) => ({
    id: column.key,
    header: column.label,
    accessorKey: column.key,
    cell: (context) => {
      const value = String(context.getValue() ?? "");
      const isPath = column.key === "source" || column.key === "destination";
      const error = context.row.original.error;
      const isFailedStatus = column.key === "status" && error !== "";
      return (
        <div
          className={`truncate px-2 py-1 ${isPath ? "font-mono text-slate-400" : ""} ${isFailedStatus ? "text-rose-400" : ""}`}
          title={isFailedStatus ? `${value}: ${error}` : value}
        >
          {value}
        </div>
      );
    }
  })
);

export function HistoryOperationsTable({ rows }: { rows: OperationTableRow[] }) {
  return (
    <div className="max-h-40 overflow-auto border border-slate-800">
      <ResizableVirtualTable
        rows={rows}
        label="Operations in this run"
        columns={OPERATION_COLUMNS}
        columnDefs={OPERATION_COLUMN_DEFS}
        defaultColumnSizing={OPERATION_DEFAULT_SIZING}
        onAutoSizeColumns={keepDefaultSizing}
        virtualized={false}
      />
    </div>
  );
}
