import { attachPlanListeners, useBackendEvents } from "../../lib/api";
import type { MovePlan } from "../../lib/types";

interface PlanEventsOptions {
  isActive: (requestId: number) => boolean;
  addPlan: (plan: MovePlan) => void;
  addError: (fileId: string, error: string) => void;
  flushPlans: () => void;
  finishRequest: (requestId: number) => void;
  reportError: (message: string) => void;
}

// Takes streamed plans and errors of the requests in flight. A reset forgets older ones.
export function usePlanEvents(options: PlanEventsOptions) {
  const { isActive, addPlan, addError, flushPlans, finishRequest, reportError } = options;
  useBackendEvents(
    attachPlanListeners,
    {
      onFile: ({ request_id, plan }) => {
        if (isActive(request_id)) {
          addPlan(plan);
        }
      },
      onError: ({ request_id, file_id, error }) => {
        if (isActive(request_id)) {
          addError(file_id, error);
        }
      },
      onFinished: ({ request_id }) => {
        if (!isActive(request_id)) {
          return;
        }
        // Plans land before the request count drops, so planning never looks done too early.
        flushPlans();
        finishRequest(request_id);
      }
    },
    (error) => reportError(`Failed to attach plan listeners: ${String(error)}`)
  );
}
