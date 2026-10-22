import { useCallback, useRef, useState } from "react";

// Tracks the plan requests in flight by id. A reset forgets them, so late answers to a
// replaced folder neither count nor land.
export function usePlanRequests() {
  const [activePlanRequests, setActivePlanRequests] = useState(0);
  const activeIdsRef = useRef<Set<number>>(new Set());
  // Ids keep growing across resets, so an old request can't pass for a new one.
  const nextIdRef = useRef(0);

  const startRequest = useCallback(() => {
    nextIdRef.current += 1;
    activeIdsRef.current.add(nextIdRef.current);
    setActivePlanRequests(activeIdsRef.current.size);
    return nextIdRef.current;
  }, []);

  const finishRequest = useCallback((requestId: number) => {
    if (activeIdsRef.current.delete(requestId)) {
      setActivePlanRequests(activeIdsRef.current.size);
    }
  }, []);

  const isActive = useCallback((requestId: number) => activeIdsRef.current.has(requestId), []);

  const resetRequests = useCallback(() => {
    activeIdsRef.current = new Set();
    setActivePlanRequests(0);
  }, []);

  return { activePlanRequests, startRequest, finishRequest, isActive, resetRequests };
}
