import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FileIdMap } from "../../lib/fileIdMap";
import { applyFilePlaces, clearScanPlans, type PlanListeners } from "../../lib/api";
import { PLAN } from "../../test/pipelineFixtures";
import { useMovePlanning } from "./useMovePlanning";

const listeners = vi.hoisted(() => ({ plan: null as PlanListeners | null }));

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  applyFilePlaces: vi.fn(async () => undefined),
  clearScanPlans: vi.fn(async () => undefined),
  attachPlanListeners: vi.fn(async (handlers: PlanListeners) => ((listeners.plan = handlers), []))
}));

async function renderPlanning() {
  const hook = renderHook(() =>
    useMovePlanning({ scanIdRef: { current: 1 }, reportError: vi.fn() })
  );
  await waitFor(() => expect(listeners.plan).not.toBeNull());
  return hook;
}

describe("useMovePlanning", () => {
  it("waits for the places, then shows the plans once the backend settles", async () => {
    const { result } = await renderPlanning();
    expect(result.current.planning).toBe(true);

    act(() => result.current.applyFilePlaces({}));
    expect(applyFilePlaces).toHaveBeenCalledWith(1, {});
    act(() => listeners.plan?.onFiles({ scan_id: 1, plans: [PLAN], errors: [] }));
    expect(result.current.planning).toBe(true);
    act(() => listeners.plan?.onSettled({ scan_id: 1 }));

    expect(result.current.movePlansByFileId).toEqual(FileIdMap.from({ f1: PLAN }));
    expect(result.current.planning).toBe(false);
    expect(result.current.waitingForPlaces()).toBe(false);
  });

  it("drops events of another scan", async () => {
    const { result } = await renderPlanning();

    act(() => {
      listeners.plan?.onFiles({
        scan_id: 0,
        plans: [PLAN],
        errors: [{ file_id: "f2", error: "gone" }]
      });
      listeners.plan?.onSettled({ scan_id: 0 });
    });

    expect(result.current.movePlansByFileId).toEqual(FileIdMap.empty());
    expect(result.current.planErrorsByFileId).toEqual({});
  });

  it("swaps a plan for its error and back", async () => {
    const { result } = await renderPlanning();

    act(() => {
      listeners.plan?.onFiles({ scan_id: 1, plans: [PLAN], errors: [] });
      listeners.plan?.onFiles({
        scan_id: 1,
        plans: [],
        errors: [{ file_id: "f1", error: "Bad date" }]
      });
      listeners.plan?.onSettled({ scan_id: 1 });
    });
    expect(result.current.planErrorsByFileId).toEqual({ f1: "Bad date" });
    expect(result.current.movePlansByFileId.has("f1")).toBe(false);

    act(() => {
      listeners.plan?.onFiles({ scan_id: 1, plans: [PLAN], errors: [] });
      listeners.plan?.onSettled({ scan_id: 1 });
    });
    expect(result.current.planErrorsByFileId).toEqual({});
  });

  it("clears the plans for a new destination and waits for the places again", async () => {
    const { result } = await renderPlanning();
    act(() => {
      result.current.applyFilePlaces({});
      listeners.plan?.onFiles({ scan_id: 1, plans: [PLAN], errors: [] });
      listeners.plan?.onSettled({ scan_id: 1 });
    });

    act(() => result.current.clearPlans());

    expect(clearScanPlans).toHaveBeenCalledWith(1);
    expect(result.current.movePlansByFileId).toEqual(FileIdMap.empty());
    expect(result.current.waitingForPlaces()).toBe(true);
    expect(result.current.planning).toBe(true);
  });
});
