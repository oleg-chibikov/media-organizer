import { Button } from "../../components/design-system/Button";
import { Pane } from "../../components/design-system/Pane";
import type { HistoryIndexEntry, HistoryRunDetails } from "../../lib/types";
import { HistoryRunDetailsSection } from "./HistoryRunDetailsSection";
import { RecentRunsTable } from "./RecentRunsTable";
import type { RunActionHandlers } from "./RunActions";

interface HistoryMenuProps extends RunActionHandlers {
  runs: HistoryIndexEntry[];
  selectedRunId: string | null;
  loading: boolean;
  details: HistoryRunDetails | null;
  detailsLoading: boolean;
  detailsError: string | null;
  onSelectRun: (runId: string) => void;
}

export function HistoryMenu({
  runs,
  loading,
  details,
  detailsLoading,
  detailsError,
  ...tableProps
}: HistoryMenuProps) {
  const showDetails = tableProps.selectedRunId && (details || detailsLoading || detailsError);

  return (
    <Pane
      panelClassName="w-[48rem] p-2"
      renderTrigger={({ open, toggle }) => (
        <Button
          variant="secondary"
          aria-expanded={open}
          className="px-3 py-1.5 text-sm"
          onClick={toggle}
        >
          History
        </Button>
      )}
    >
      <p className="mb-1 text-xs text-slate-400">Recent Runs</p>
      <div className="max-h-56 overflow-y-auto">
        {loading ? (
          <p className="px-2 py-1 text-xs text-slate-500">Loading history...</p>
        ) : runs.length === 0 ? (
          <p className="px-2 py-1 text-xs text-slate-500">No history runs</p>
        ) : (
          <RecentRunsTable runs={runs} {...tableProps} />
        )}
      </div>
      {showDetails ? (
        <HistoryRunDetailsSection details={details} loading={detailsLoading} error={detailsError} />
      ) : null}
    </Pane>
  );
}
