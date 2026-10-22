import { useCallback, useEffect, useRef, useState } from "react";
import type { MovePlan } from "../../lib/types";
import { FileIdMap } from "../../lib/fileIdMap";
import { withoutFileIds } from "./withoutFileIds";

const fileIdOf = (plan: MovePlan) => plan.file_id;

// Each flush redraws every row, so a large folder needs few of them. Still feels live.
const PLAN_FLUSH_MS = 250;

// Collects streamed plans and applies them in batches, one render per batch instead of per file.
export function usePlanBatches() {
  const [movePlansByFileId, setMovePlansByFileId] = useState(FileIdMap.empty<MovePlan>);
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
    setMovePlansByFileId((previous) => previous.with(Object.values(pending), fileIdOf));
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
    setMovePlansByFileId(FileIdMap.empty());
  }, [cancelScheduledFlush]);

  const dropPlansFor = useCallback((fileIds: string[]) => {
    pendingRef.current = withoutFileIds(pendingRef.current, fileIds);
    setMovePlansByFileId((previous) => previous.without(fileIds));
  }, []);

  useEffect(() => cancelScheduledFlush, [cancelScheduledFlush]);

  return { movePlansByFileId, addPlan, flushPlans, dropPlans, dropPlansFor };
}
