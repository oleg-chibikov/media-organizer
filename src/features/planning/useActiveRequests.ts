import { useCallback, useRef, useState } from "react";

// Counts plan requests in flight. The ref keeps the count right between renders.
export function useActiveRequests() {
  const [activePlanRequests, setActivePlanRequests] = useState(0);
  const activeRequestsRef = useRef(0);

  const changeActiveRequests = useCallback((delta: number) => {
    activeRequestsRef.current = Math.max(0, activeRequestsRef.current + delta);
    setActivePlanRequests(activeRequestsRef.current);
  }, []);

  const resetActiveRequests = useCallback(() => {
    activeRequestsRef.current = 0;
    setActivePlanRequests(0);
  }, []);

  return { activePlanRequests, changeActiveRequests, resetActiveRequests };
}
