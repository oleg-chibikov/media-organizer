import type { ColumnDef } from "@tanstack/react-table";
import { useMemo } from "react";
import { ResizableVirtualTable } from "../../components/design-system/ResizableVirtualTable";
import type { HistoryIndexEntry } from "../../lib/types";
import {
  buildRecentRunRows,
  RECENT_RUN_COLUMNS,
  RECENT_RUN_DEFAULT_SIZING,
  type RecentRunTableRow
} from "./recentRunRows";
import { RunActions, type RunActionHandlers } from "./RunActions";

interface RecentRunsTableProps extends RunActionHandlers {
  runs: HistoryIndexEntry[];
  selectedRunId: string | null;
  onSelectRun: (runId: string) => void;
}

const keepDefaultSizing = () => RECENT_RUN_DEFAULT_SIZING;

export function RecentRunsTable({
  runs,
  selectedRunId,
  onSelectRun,
  onUndoRun,
  onRedoRun,
  onDeleteRun,
  onOpenLog
}: RecentRunsTableProps) {
  const rows = useMemo(() => buildRecentRunRows(runs), [runs]);

  const columnDefs = useMemo<ColumnDef<RecentRunTableRow>[]>(
    () =>
      RECENT_RUN_COLUMNS.map((column) => ({
        id: column.key,
        header: column.label,
        accessorKey: column.key,
        cell: (context) => {
          if (column.key === "actions") {
            const handlers = { onUndoRun, onRedoRun, onDeleteRun, onOpenLog };
            return <RunActions run={context.row.original.run} {...handlers} />;
          }
          const value = String(context.getValue() ?? "");
          const mono = column.key === "date" || column.key === "migrated";
          const muted = column.key === "status";
          return (
            <div
              className={`w-full truncate px-2 py-1 text-left text-xs ${mono ? "font-mono text-[11px]" : ""} ${muted ? "text-[11px] text-slate-400" : ""}`}
              title={value}
            >
              {value}
            </div>
          );
        }
      })),
    [onDeleteRun, onOpenLog, onRedoRun, onUndoRun]
  );

  return (
    <ResizableVirtualTable
      rows={rows}
      columns={RECENT_RUN_COLUMNS}
      columnDefs={columnDefs}
      defaultColumnSizing={RECENT_RUN_DEFAULT_SIZING}
      onAutoSizeColumns={keepDefaultSizing}
      selectable={true}
      onRowClick={(row) => onSelectRun(row.run.run_id)}
      getRowAriaLabel={(row) => `View run ${row.run.run_id}`}
      virtualized={false}
      getRowClassName={(row) =>
        selectedRunId === row.run.run_id
          ? "bg-sky-800 text-sky-100 hover:bg-sky-700"
          : "text-slate-300 hover:bg-slate-800"
      }
    />
  );
}
