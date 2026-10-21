import { useCallback, useEffect, useRef, useState, type RefObject } from "react";
import { getMoveConfirmationSummary } from "../../lib/api";
import type {
  FileRecord,
  MetadataResult,
  MoveConfirmationSummary,
  MovePlan
} from "../../lib/types";
import { collectConfirmationInput } from "./moveConfirmationPrompt";

interface MoveConfirmationOptions {
  rows: FileRecord[];
  rowByIdRef: RefObject<Map<string, FileRecord>>;
  metadataByFileId: Record<string, MetadataResult>;
  movePlansByFileId: Record<string, MovePlan>;
  reportError: (message: string) => void;
  onConfirmed: () => void;
}

interface PendingConfirmation {
  summary: MoveConfirmationSummary;
  // A new scan swaps the row index; its files were not in the summary the person saw.
  scanSession: Map<string, FileRecord>;
}

// Gets the backend's own count of what it would move, then waits for the person to answer.
export function useMoveConfirmation({
  rows,
  rowByIdRef,
  metadataByFileId,
  movePlansByFileId,
  reportError,
  onConfirmed
}: MoveConfirmationOptions) {
  const [pending, setPending] = useState<PendingConfirmation | null>(null);
  const askingRef = useRef(false);
  const onConfirmedRef = useRef(onConfirmed);
  useEffect(() => {
    onConfirmedRef.current = onConfirmed;
  });

  const request = async () => {
    if (askingRef.current || pending) {
      return;
    }
    askingRef.current = true;
    const scanSession = rowByIdRef.current;
    try {
      const input = collectConfirmationInput(rows, metadataByFileId, movePlansByFileId);
      const summary = await getMoveConfirmationSummary(input.files, input.metadata, input.plans);
      if (rowByIdRef.current === scanSession) {
        setPending({ summary, scanSession });
      }
    } catch (error) {
      reportError(`Failed to prepare the move summary: ${String(error)}`);
    } finally {
      askingRef.current = false;
    }
  };

  const confirm = () => {
    setPending(null);
    if (pending && rowByIdRef.current === pending.scanSession) {
      onConfirmedRef.current();
    }
  };

  const cancel = useCallback(() => setPending(null), []);

  return { summary: pending?.summary ?? null, request, confirm, cancel };
}
