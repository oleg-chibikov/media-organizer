import type { HistoryActionKind } from "./historyActionProgress";

export type HistoryRunAction = HistoryActionKind | "delete";

const PROMPTS: Record<HistoryRunAction, string> = {
  undo: "Undo this run? Moved files go back to their original folders. Copies are removed only when the original still holds the same bytes.",
  redo: "Redo this run? Files go to their processed folders again. Existing files are never replaced.",
  delete:
    "Delete this history entry? You can't undo or redo this run afterwards. Media files stay where they are."
};

// Asks before any history action that moves files or drops the ability to undo them.
export function confirmHistoryAction(
  action: HistoryRunAction,
  confirm: (message: string) => boolean = (message) => window.confirm(message)
): boolean {
  return confirm(PROMPTS[action]);
}
