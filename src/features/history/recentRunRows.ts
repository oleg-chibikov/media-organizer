import type { ColumnSizingState } from "@tanstack/react-table";
import type { HistoryIndexEntry } from "../../lib/types";
import { formatHistoryTimestamp } from "./formatHistoryTimestamp";
import { historyDestinationRoot } from "./historyPaths";
import { formatRunStatus } from "./historyStatusLabels";

// Actions sit before the To path, so Undo stays in view without scrolling sideways.
export const RECENT_RUN_COLUMNS = [
  { key: "date", label: "When" },
  { key: "source", label: "From" },
  { key: "migrated", label: "Files" },
  { key: "status", label: "Status" },
  { key: "actions", label: "Actions" },
  { key: "destination", label: "To" }
] as const;

type RecentRunColumnKey = (typeof RECENT_RUN_COLUMNS)[number]["key"];
export type RecentRunTableRow = { id: string; run: HistoryIndexEntry } & Record<
  RecentRunColumnKey,
  string
>;

export const RECENT_RUN_DEFAULT_SIZING: ColumnSizingState = {
  date: 164,
  source: 220,
  migrated: 64,
  status: 140,
  actions: 170,
  destination: 240
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
