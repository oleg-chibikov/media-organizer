import { useEffect, useRef, type RefObject } from "react";
import { getMoveConfirmationSummary } from "../../lib/api";
import type { FileRecord, MetadataResult, MovePlan } from "../../lib/types";
import { buildMoveConfirmationPrompt, collectConfirmationInput } from "./moveConfirmationPrompt";

interface MoveConfirmationOptions {
  rows: FileRecord[];
  rowByIdRef: RefObject<Map<string, FileRecord>>;
  metadataByFileId: Record<string, MetadataResult>;
  movePlansByFileId: Record<string, MovePlan>;
  reportError: (message: string) => void;
  onConfirmed: () => void;
}

// Shows the backend's own count of what it would move, then asks before anything moves.
export function useMoveConfirmation({
  rows,
  rowByIdRef,
  metadataByFileId,
  movePlansByFileId,
  reportError,
  onConfirmed
}: MoveConfirmationOptions) {
  const askingRef = useRef(false);
  const onConfirmedRef = useRef(onConfirmed);
  useEffect(() => {
    onConfirmedRef.current = onConfirmed;
  });

  return async () => {
    if (askingRef.current) {
      return;
    }
    askingRef.current = true;
    // A new scan swaps the row index; its files were not in the summary the person saw.
    const scanSession = rowByIdRef.current;
    try {
      const input = collectConfirmationInput(rows, metadataByFileId, movePlansByFileId);
      const summary = await getMoveConfirmationSummary(input.files, input.metadata, input.plans);
      if (
        rowByIdRef.current === scanSession &&
        window.confirm(buildMoveConfirmationPrompt(summary))
      ) {
        onConfirmedRef.current();
      }
    } catch (error) {
      reportError(`Failed to prepare the move summary: ${String(error)}`);
    } finally {
      askingRef.current = false;
    }
  };
}
