import { Button } from "../../components/design-system/Button";
import { Dialog } from "../../components/design-system/Dialog";
import type { MoveConfirmationSummary } from "../../lib/types";
import { fileCountLabel, moveConfirmationNotes } from "./moveConfirmationPrompt";

interface MoveConfirmationDialogProps {
  summary: MoveConfirmationSummary;
  destinationRoot: string | null;
  // The summary counts only files checked so far; the rest join the run as they become ready.
  stillChecking: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function MoveConfirmationDialog({
  summary,
  destinationRoot,
  stillChecking,
  onConfirm,
  onCancel
}: MoveConfirmationDialogProps) {
  const ready = fileCountLabel(summary.ready_operations);
  const files = stillChecking ? "all files" : ready;
  const notes = moveConfirmationNotes(summary);

  return (
    <Dialog
      title={`Move ${files}?`}
      onCancel={onCancel}
      actions={
        <>
          <Button variant="secondary" className="px-3 py-1.5 text-sm" onClick={onCancel}>
            Cancel
          </Button>
          <Button variant="primary" className="px-3 py-1.5 text-sm" onClick={onConfirm}>
            Move {files}
          </Button>
        </>
      }
    >
      <p className="text-slate-400">To</p>
      <p className="break-all font-mono text-xs text-slate-100">
        {destinationRoot ?? "The destination in the toolbar"}
      </p>
      <p className="mt-3">
        Each file goes into a folder for its year, month and day. Its new name starts with the
        capture time.
      </p>
      {notes.length > 0 ? (
        <ul className="mt-3 list-disc space-y-1 pl-5 text-amber-200">
          {notes.map((note) => (
            <li key={note}>{note}</li>
          ))}
        </ul>
      ) : null}
      {stillChecking ? (
        <p className="mt-3 text-amber-200">
          {ready} ready now. Files still being checked move too when they are ready.
        </p>
      ) : null}
      <p className="mt-3 text-slate-400">You can undo this run in History.</p>
    </Dialog>
  );
}
