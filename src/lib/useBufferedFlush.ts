import { useCallback, useEffect, useRef } from "react";

// Collects items from a stream of backend events and applies them in one call per interval,
// so a burst of thousands of events costs a few renders instead of one each.
export function useBufferedFlush<T>(apply: (items: T[]) => void, intervalMs: number) {
  const applyRef = useRef(apply);
  const pendingRef = useRef<T[]>([]);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    applyRef.current = apply;
  });

  const cancelTimer = useCallback(() => {
    if (timerRef.current !== null) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const flush = useCallback(() => {
    cancelTimer();
    const items = pendingRef.current;
    if (items.length > 0) {
      pendingRef.current = [];
      applyRef.current(items);
    }
  }, [cancelTimer]);

  const add = useCallback(
    (item: T) => {
      pendingRef.current.push(item);
      timerRef.current ??= setTimeout(flush, intervalMs);
    },
    [flush, intervalMs]
  );

  const drop = useCallback(() => {
    cancelTimer();
    pendingRef.current = [];
  }, [cancelTimer]);

  useEffect(() => cancelTimer, [cancelTimer]);

  return { add, flush, drop };
}
