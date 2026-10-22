import { useCallback, useState } from "react";
import type { MovePlan, PlanFileError } from "../../lib/types";
import { FileIdMap } from "../../lib/fileIdMap";
import { useBufferedFlush } from "../../lib/useBufferedFlush";
import { withoutFileIds } from "./withoutFileIds";

export interface PlannedFiles {
  plans: MovePlan[];
  errors: PlanFileError[];
}

const fileIdOf = (plan: MovePlan) => plan.file_id;

// Each flush redraws every row, so a large folder needs few of them. Still feels live.
const PLAN_FLUSH_MS = 250;

// The last plan or error of each file, since a later pass replaces an earlier one.
function latestOutcomes(batches: PlannedFiles[]) {
  const latest = new Map<string, MovePlan | PlanFileError>();
  for (const { plans, errors } of batches) {
    for (const plan of plans) {
      latest.set(plan.file_id, plan);
    }
    for (const failed of errors) {
      latest.set(failed.file_id, failed);
    }
  }
  const plans: MovePlan[] = [];
  const errors: Record<string, string> = {};
  for (const outcome of latest.values()) {
    if ("error" in outcome) {
      errors[outcome.file_id] = outcome.error;
    } else {
      plans.push(outcome);
    }
  }
  return { plans, errors };
}

// Collects streamed plans and errors and applies them in batches, one render per batch.
export function usePlanBatches() {
  const [movePlansByFileId, setMovePlansByFileId] = useState(FileIdMap.empty<MovePlan>);
  const [planErrorsByFileId, setPlanErrorsByFileId] = useState<Record<string, string>>({});

  const apply = useCallback((batches: PlannedFiles[]) => {
    const { plans, errors } = latestOutcomes(batches);
    const failedIds = Object.keys(errors);
    setMovePlansByFileId((previous) => previous.with(plans, fileIdOf).without(failedIds));
    setPlanErrorsByFileId((previous) => {
      const next = withoutFileIds(previous, plans.map(fileIdOf));
      return failedIds.length > 0 ? { ...next, ...errors } : next;
    });
  }, []);
  const { add, flush, drop } = useBufferedFlush(apply, PLAN_FLUSH_MS);

  const dropPlans = useCallback(() => {
    drop();
    setMovePlansByFileId(FileIdMap.empty());
    setPlanErrorsByFileId({});
  }, [drop]);

  return {
    movePlansByFileId,
    planErrorsByFileId,
    addPlanned: add,
    flushPlans: flush,
    dropPlans
  };
}
