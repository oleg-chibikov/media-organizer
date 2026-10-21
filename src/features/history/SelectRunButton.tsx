import type { HistoryIndexEntry } from "../../lib/types";

interface SelectRunButtonProps {
  run: HistoryIndexEntry;
  selected: boolean;
  onSelectRun: (runId: string) => void;
}

// The keyboard way to open a run's details; a click anywhere on the row does the same.
export function SelectRunButton({ run, selected, onSelectRun }: SelectRunButtonProps) {
  return (
    <button
      type="button"
      aria-current={selected ? "true" : undefined}
      className="w-full cursor-pointer truncate px-2 py-1 text-left text-xs underline-offset-2 hover:underline"
      title={`View run from ${run.selected_folder}`}
      onClick={(event) => {
        event.stopPropagation();
        onSelectRun(run.run_id);
      }}
    >
      {run.selected_folder}
    </button>
  );
}
