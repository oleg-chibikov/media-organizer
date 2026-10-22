import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FileIdMap } from "../../lib/fileIdMap";
import { applyPlaceSuggestions, clearScanPlans, type PlanListeners } from "../../lib/api";
import type { PlacedMove } from "../../lib/types";
import { METADATA, PLAN, RECORD } from "../../test/pipelineFixtures";
import { useMovePlanning } from "./useMovePlanning";

const listeners = vi.hoisted(() => ({ plan: null as PlanListeners | null }));

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  applyPlaceSuggestions: vi.fn(async () => undefined),
  clearScanPlans: vi.fn(async () => undefined),
  attachPlanListeners: vi.fn(async (handlers: PlanListeners) => ((listeners.plan = handlers), []))
}));

async function renderPlanning() {
  const onPlacedFiles = vi.fn();
  const hook = renderHook(() =>
    useMovePlanning({ scanIdRef: { current: 1 }, reportError: vi.fn(), onPlacedFiles })
  );
  await waitFor(() => expect(listeners.plan).not.toBeNull());
  return { ...hook, onPlacedFiles };
}

describe("useMovePlanning", () => {
  it("waits for the places, then shows the plans once the backend settles", async () => {
    const { result } = await renderPlanning();
    expect(result.current.planning).toBe(true);

    act(() => result.current.applyPlaceSuggestions([]));
    expect(applyPlaceSuggestions).toHaveBeenCalledWith(1, []);
    act(() => listeners.plan?.onFile({ scan_id: 1, plan: PLAN }));
    expect(result.current.planning).toBe(true);
    act(() => listeners.plan?.onSettled({ scan_id: 1 }));

    expect(result.current.movePlansByFileId).toEqual(FileIdMap.from({ f1: PLAN }));
    expect(result.current.planning).toBe(false);
    expect(result.current.waitingForPlaces()).toBe(false);
  });

  it("drops events of another scan", async () => {
    const { result } = await renderPlanning();

    act(() => {
      listeners.plan?.onFile({ scan_id: 0, plan: PLAN });
      listeners.plan?.onError({ scan_id: 0, file_id: "f1", error: "gone" });
      listeners.plan?.onSettled({ scan_id: 0 });
    });

    expect(result.current.movePlansByFileId).toEqual(FileIdMap.empty());
    expect(result.current.planErrorsByFileId).toEqual({});
  });

  it("swaps a plan for its error and back", async () => {
    const { result } = await renderPlanning();

    act(() => {
      listeners.plan?.onFile({ scan_id: 1, plan: PLAN });
      listeners.plan?.onError({ scan_id: 1, file_id: "f1", error: "Bad date" });
      listeners.plan?.onSettled({ scan_id: 1 });
    });
    expect(result.current.planErrorsByFileId).toEqual({ f1: "Bad date" });
    expect(result.current.movePlansByFileId.has("f1")).toBe(false);

    act(() => {
      listeners.plan?.onFile({ scan_id: 1, plan: PLAN });
      listeners.plan?.onSettled({ scan_id: 1 });
    });
    expect(result.current.planErrorsByFileId).toEqual({});
  });

  it("adds placed files with their plans and drops them again", async () => {
    const { result, onPlacedFiles } = await renderPlanning();
    const placed: PlacedMove = { record: RECORD, metadata: METADATA, plan: PLAN };

    act(() => {
      listeners.plan?.onPlaced({ scan_id: 1, added: [placed], dropped: [] });
      listeners.plan?.onSettled({ scan_id: 1 });
    });
    expect(onPlacedFiles).toHaveBeenCalledWith([placed], []);
    expect(result.current.movePlansByFileId.get("f1")).toEqual(PLAN);

    act(() => listeners.plan?.onPlaced({ scan_id: 1, added: [], dropped: ["f1"] }));
    expect(result.current.movePlansByFileId.has("f1")).toBe(false);
  });

  it("clears the plans for a new destination and waits for the places again", async () => {
    const { result } = await renderPlanning();
    act(() => {
      result.current.applyPlaceSuggestions([]);
      listeners.plan?.onFile({ scan_id: 1, plan: PLAN });
      listeners.plan?.onSettled({ scan_id: 1 });
    });

    act(() => result.current.clearPlans());

    expect(clearScanPlans).toHaveBeenCalledWith(1);
    expect(result.current.movePlansByFileId).toEqual(FileIdMap.empty());
    expect(result.current.waitingForPlaces()).toBe(true);
    expect(result.current.planning).toBe(true);
  });
});
