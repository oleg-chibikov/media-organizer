import { useCallback, useEffect, useRef, useState } from "react";
import type { MovePlan } from "../../lib/types";

// Long enough to batch a burst of plan events, short enough to feel live.
const PLAN_FLUSH_MS = 50;

// Collects streamed plans and applies them in batches, one render per batch instead of per file.
export function usePlanBatches() {
  const [movePlansByFileId, setMovePlansByFileId] = useState<Record<string, MovePlan>>({});
  const pendingRef = useRef<Record<string, MovePlan>>({});
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const cancelScheduledFlush = useCallback(() => {
    if (flushTimerRef.current !== null) {
      clearTimeout(flushTimerRef.current);
      flushTimerRef.current = null;
    }
  }, []);

  const flushPlans = useCallback(() => {
    cancelScheduledFlush();
    const pending = pendingRef.current;
    if (Object.keys(pending).length === 0) {
      return;
    }
    pendingRef.current = {};
    setMovePlansByFileId((previous) => ({ ...previous, ...pending }));
  }, [cancelScheduledFlush]);

  const addPlan = useCallback(
    (plan: MovePlan) => {
      pendingRef.current[plan.file_id] = plan;
      flushTimerRef.current ??= setTimeout(flushPlans, PLAN_FLUSH_MS);
    },
    [flushPlans]
  );

  const dropPlans = useCallback(() => {
    cancelScheduledFlush();
    pendingRef.current = {};
    setMovePlansByFileId({});
  }, [cancelScheduledFlush]);

  useEffect(() => cancelScheduledFlush, [cancelScheduledFlush]);

  return { movePlansByFileId, addPlan, flushPlans, dropPlans };
}
