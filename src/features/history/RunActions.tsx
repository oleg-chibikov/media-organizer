import { Button } from "../../components/design-system/Button";
import type { HistoryIndexEntry } from "../../lib/types";

export interface RunActionHandlers {
  onUndoRun: (runId: string) => void;
  onRedoRun: (runId: string) => void;
  onDeleteRun: (runId: string) => void;
  onOpenLog: (historyFilePath: string) => void;
}

const ACTION_CLASS = "px-2 py-1 text-[11px]";

export function RunActions({
  run,
  onUndoRun,
  onRedoRun,
  onDeleteRun,
  onOpenLog
}: RunActionHandlers & { run: HistoryIndexEntry }) {
  return (
    <div
      className="flex justify-start gap-1 px-2 py-1"
      onClick={(event) => event.stopPropagation()}
    >
      {run.undo_available ? (
        <Button
          variant="secondary"
          aria-label={`Undo ${run.folder_label}`}
          className={ACTION_CLASS}
          onClick={() => onUndoRun(run.run_id)}
        >
          Undo
        </Button>
      ) : run.redo_available ? (
        <Button
          variant="secondary"
          aria-label={`Redo ${run.folder_label}`}
          className={ACTION_CLASS}
          onClick={() => onRedoRun(run.run_id)}
        >
          Redo
        </Button>
      ) : null}
      <Button
        variant="secondary"
        aria-label={`Open log for ${run.folder_label}`}
        className={ACTION_CLASS}
        onClick={() => onOpenLog(run.history_file_path)}
      >
        Log
      </Button>
      <Button
        variant="secondary"
        aria-label={`Delete ${run.folder_label}`}
        className="px-2 py-1 text-[11px] text-red-400 hover:bg-red-900 hover:text-red-300"
        onClick={() => onDeleteRun(run.run_id)}
      >
        Delete
      </Button>
    </div>
  );
}
