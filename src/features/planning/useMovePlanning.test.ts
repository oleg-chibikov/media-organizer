import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { PlanListeners } from "../../lib/api";
import { METADATA, PLAN, RECORD } from "../../test/pipelineFixtures";
import { useMovePlanning } from "./useMovePlanning";

const listeners = vi.hoisted(() => ({ plan: null as PlanListeners | null }));

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  createMovePlan: vi.fn(async () => undefined),
  attachPlanListeners: vi.fn(async (handlers: PlanListeners) => ((listeners.plan = handlers), []))
}));

function renderPlanning() {
  const reportError = vi.fn();
  const hook = renderHook(() => useMovePlanning({ selectedFolder: "C:\\in", reportError }));
  return { ...hook, reportError };
}

describe("useMovePlanning", () => {
  it("applies requested plans in one batch when planning finishes", async () => {
    const { result } = renderPlanning();
    await waitFor(() => expect(listeners.plan).not.toBeNull());

    act(() => result.current.requestPlans([RECORD], [METADATA]));
    act(() => listeners.plan?.onFile({ plan: PLAN }));
    expect(result.current.movePlansByFileId).toEqual({});
    act(() => listeners.plan?.onFinished({ planned: 1, errors: 0 }));

    expect(result.current.movePlansByFileId).toEqual({ f1: PLAN });
    expect(result.current.activePlanRequests).toBe(0);
  });

  it("ignores plans and errors for files it did not request", async () => {
    const { result, reportError } = renderPlanning();
    await waitFor(() => expect(listeners.plan).not.toBeNull());

    act(() => {
      listeners.plan?.onFile({ plan: { ...PLAN, file_id: "old" } });
      listeners.plan?.onError({ file_id: "old", error: "gone" });
      listeners.plan?.onFinished({ planned: 1, errors: 1 });
    });

    expect(result.current.movePlansByFileId).toEqual({});
    expect(result.current.planErrorsByFileId).toEqual({});
    expect(reportError).not.toHaveBeenCalled();
  });

  it("keeps a plan error on its file until the file is planned again", async () => {
    const { result, reportError } = renderPlanning();
    await waitFor(() => expect(listeners.plan).not.toBeNull());

    act(() => result.current.requestPlans([RECORD], [METADATA]));
    act(() => listeners.plan?.onError({ file_id: RECORD.id, error: "Bad date" }));
    expect(result.current.planErrorsByFileId).toEqual({ f1: "Bad date" });
    expect(reportError).not.toHaveBeenCalled();

    act(() => result.current.requestPlans([RECORD], [{ ...METADATA, chosen_date: "2026-05-25" }]));
    expect(result.current.planErrorsByFileId).toEqual({});
  });
});
