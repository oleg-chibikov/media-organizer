import { attachPlanListeners, useBackendEvents } from "../../lib/api";
import type { MovePlan } from "../../lib/types";

interface PlanEventsOptions {
  isRequested: (fileId: string) => boolean;
  addPlan: (plan: MovePlan) => void;
  addError: (fileId: string, error: string) => void;
  flushPlans: () => void;
  changeActiveRequests: (delta: number) => void;
  reportError: (message: string) => void;
}

// Takes streamed plans and errors for the files this session asked about.
export function usePlanEvents(options: PlanEventsOptions) {
  const { isRequested, addPlan, addError, flushPlans, changeActiveRequests, reportError } = options;
  useBackendEvents(
    attachPlanListeners,
    {
      onFile: ({ plan }) => {
        if (isRequested(plan.file_id)) {
          addPlan(plan);
        }
      },
      onError: ({ file_id, error }) => {
        if (isRequested(file_id)) {
          addError(file_id, error);
        }
      },
      onFinished: () => {
        // Plans land before the request count drops, so planning never looks done too early.
        flushPlans();
        changeActiveRequests(-1);
      }
    },
    (error) => reportError(`Failed to attach plan listeners: ${String(error)}`)
  );
}
