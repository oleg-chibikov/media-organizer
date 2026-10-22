import { useCallback, useEffect, useRef } from "react";
import { planContinuedMoves } from "../../lib/api";
import type { MetadataResult, MovePlan, PlaceSuggestion, PlacedMove } from "../../lib/types";

export interface PlacedMovesOptions {
  selectedFolder: string | null;
  addPlan: (plan: MovePlan) => void;
  flushPlans: () => void;
  dropPlansFor: (fileIds: string[]) => void;
  startRequest: () => number;
  finishRequest: (requestId: number) => void;
  reportError: (message: string) => void;
  // Placed files that move to new day folders join the rows, and leave them when dropped.
  onPlacedFiles: (added: PlacedMove[], droppedIds: string[]) => void;
}

// Plans the files earlier runs placed in the folders the accepted suggestions continue.
export function usePlacedMoves(options: PlacedMovesOptions) {
  const optionsRef = useRef(options);
  useEffect(() => {
    optionsRef.current = options;
  });
  // Every placed file this scan has seen. Normal planning would send them to date folders.
  const placedIdsRef = useRef<Set<string>>(new Set());
  const activeIdsRef = useRef<string[]>([]);
  const requestRef = useRef(0);

  const dropPlaced = useCallback(() => {
    requestRef.current += 1;
    const dropped = activeIdsRef.current;
    activeIdsRef.current = [];
    if (dropped.length > 0) {
      optionsRef.current.dropPlansFor(dropped);
      optionsRef.current.onPlacedFiles([], dropped);
    }
  }, []);

  const planPlaced = useCallback(
    (suggestions: PlaceSuggestion[], metadataResults: MetadataResult[]) => {
      dropPlaced();
      const { selectedFolder, startRequest } = optionsRef.current;
      if (!selectedFolder || !suggestions.some((suggestion) => suggestion.continues)) {
        return;
      }
      const request = requestRef.current;
      const isCurrent = () => request === requestRef.current;
      const requestId = startRequest();
      planContinuedMoves(selectedFolder, suggestions, metadataResults)
        .then((moves) => {
          if (!isCurrent()) {
            return;
          }
          const latest = optionsRef.current;
          moves.forEach((move) => placedIdsRef.current.add(move.record.id));
          activeIdsRef.current = moves.map((move) => move.record.id);
          latest.onPlacedFiles(moves, []);
          moves.forEach((move) => latest.addPlan(move.plan));
          latest.flushPlans();
        })
        .catch((error: unknown) => {
          if (isCurrent()) {
            optionsRef.current.reportError(`Failed to plan placed files: ${String(error)}`);
          }
        })
        .finally(() => optionsRef.current.finishRequest(requestId));
    },
    [dropPlaced]
  );

  const isPlaced = useCallback((fileId: string) => placedIdsRef.current.has(fileId), []);

  const resetPlaced = useCallback(() => {
    requestRef.current += 1;
    placedIdsRef.current = new Set();
    activeIdsRef.current = [];
  }, []);

  return { planPlaced, dropPlaced, isPlaced, resetPlaced };
}
