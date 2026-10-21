import type { ColumnSizingState } from "@tanstack/react-table";
import type { HistoryIndexEntry } from "../../lib/types";
import { formatHistoryTimestamp } from "./formatHistoryTimestamp";
import { historyDestinationRoot } from "./historyPaths";
import { formatRunStatus } from "./historyStatusLabels";

export const RECENT_RUN_COLUMNS = [
  { key: "source", label: "Source Path" },
  { key: "destination", label: "Destination Path" },
  { key: "migrated", label: "Files Migrated" },
  { key: "date", label: "Date / Time" },
  { key: "status", label: "Status" },
  { key: "actions", label: "Actions" }
] as const;

export type RecentRunColumnKey = (typeof RECENT_RUN_COLUMNS)[number]["key"];
export type RecentRunTableRow = { id: string; run: HistoryIndexEntry } & Record<
  RecentRunColumnKey,
  string
>;

export const RECENT_RUN_DEFAULT_SIZING: ColumnSizingState = {
  source: 240,
  destination: 240,
  migrated: 110,
  date: 170,
  status: 140,
  actions: 210
};

export function buildRecentRunRows(runs: HistoryIndexEntry[]): RecentRunTableRow[] {
  return runs.map((run) => ({
    id: run.run_id,
    run,
    source: run.selected_folder,
    destination: historyDestinationRoot(run),
    migrated: String(run.files_migrated ?? 0),
    date: formatHistoryTimestamp(run.started_at),
    status: formatRunStatus(run.run_status),
    actions: "actions"
  }));
}
