import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { createMovePlan, type PlanListeners } from "../../lib/api";
import type { PlaceSuggestion } from "../../lib/types";
import { METADATA, PLAN, RECORD } from "../../test/pipelineFixtures";
import { useMovePlanning } from "./useMovePlanning";

const listeners = vi.hoisted(() => ({ plan: null as PlanListeners | null }));

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  createMovePlan: vi.fn(async () => undefined),
  attachPlanListeners: vi.fn(async (handlers: PlanListeners) => ((listeners.plan = handlers), []))
}));

const lastRequestId = () => vi.mocked(createMovePlan).mock.lastCall?.[0] ?? 0;

function renderPlanning() {
  const reportError = vi.fn();
  const hook = renderHook(() =>
    useMovePlanning({ selectedFolder: "C:\\in", reportError, onPlacedFiles: vi.fn() })
  );
  return { ...hook, reportError };
}

describe("useMovePlanning", () => {
  it("applies requested plans in one batch when planning finishes", async () => {
    const { result } = renderPlanning();
    await waitFor(() => expect(listeners.plan).not.toBeNull());

    act(() => result.current.requestPlans([RECORD], [METADATA]));
    const request_id = lastRequestId();
    act(() => listeners.plan?.onFile({ request_id, plan: PLAN }));
    expect(result.current.movePlansByFileId).toEqual({});
    act(() => listeners.plan?.onFinished({ request_id, planned: 1, errors: 0 }));

    expect(result.current.movePlansByFileId).toEqual({ f1: PLAN });
    expect(result.current.activePlanRequests).toBe(0);
  });

  it("ignores plans and errors of requests it did not make", async () => {
    const { result, reportError } = renderPlanning();
    await waitFor(() => expect(listeners.plan).not.toBeNull());

    act(() => {
      listeners.plan?.onFile({ request_id: -1, plan: { ...PLAN, file_id: "old" } });
      listeners.plan?.onError({ request_id: -1, file_id: "old", error: "gone" });
      listeners.plan?.onFinished({ request_id: -1, planned: 1, errors: 1 });
    });

    expect(result.current.movePlansByFileId).toEqual({});
    expect(result.current.planErrorsByFileId).toEqual({});
    expect(reportError).not.toHaveBeenCalled();
  });

  it("keeps a plan error on its file until the file is planned again", async () => {
    const { result, reportError } = renderPlanning();
    await waitFor(() => expect(listeners.plan).not.toBeNull());

    act(() => result.current.requestPlans([RECORD], [METADATA]));
    act(() =>
      listeners.plan?.onError({
        request_id: lastRequestId(),
        file_id: RECORD.id,
        error: "Bad date"
      })
    );
    expect(result.current.planErrorsByFileId).toEqual({ f1: "Bad date" });
    expect(reportError).not.toHaveBeenCalled();

    act(() => result.current.requestPlans([RECORD], [{ ...METADATA, chosen_date: "2026-05-25" }]));
    expect(result.current.planErrorsByFileId).toEqual({});
  });
});

describe("useMovePlanning reset", () => {
  it("drops the answers to requests made before a reset", async () => {
    const { result } = renderPlanning();
    await waitFor(() => expect(listeners.plan).not.toBeNull());
    act(() => result.current.requestPlans([RECORD], [METADATA]));
    const old = lastRequestId();

    act(() => result.current.resetPlanning());
    act(() => result.current.requestPlans([RECORD], [METADATA]));
    act(() => {
      listeners.plan?.onFile({ request_id: old, plan: { ...PLAN, destination_path: "old" } });
      listeners.plan?.onFinished({ request_id: old, planned: 1, errors: 0 });
    });

    expect(result.current.movePlansByFileId).toEqual({});
    expect(result.current.activePlanRequests).toBe(1);
  });
});

describe("useMovePlanning place folders", () => {
  const other = { ...RECORD, id: "f2" };
  const otherMetadata = { ...METADATA, file_id: "f2" };
  const outing = {
    id: "o1",
    kind: "outing",
    places: [{ name: "Gardens", file_ids: ["f1"] }],
    root_file_ids: [],
    continues: null
  } as unknown as PlaceSuggestion;

  it("plans again only the files whose place folder changed", async () => {
    const { result } = renderPlanning();
    await waitFor(() => expect(listeners.plan).not.toBeNull());
    const rows = [RECORD, other];
    const metadata = [METADATA, otherMetadata];
    act(() => result.current.requestPlans(rows, metadata));
    const request_id = lastRequestId();
    act(() => {
      listeners.plan?.onFile({ request_id, plan: PLAN });
      listeners.plan?.onFinished({ request_id, planned: 1, errors: 0 });
    });
    vi.mocked(createMovePlan).mockClear();

    act(() => result.current.applyPlaceSuggestions([outing], rows, metadata));

    expect(createMovePlan).toHaveBeenCalledExactlyOnceWith(
      expect.any(Number),
      "C:\\in",
      [RECORD],
      [METADATA],
      [outing]
    );
    expect(result.current.movePlansByFileId).toEqual({});
    expect(result.current.activePlanRequests).toBe(1);

    vi.mocked(createMovePlan).mockClear();
    act(() => result.current.applyPlaceSuggestions([], rows, metadata));
    expect(createMovePlan).toHaveBeenCalledExactlyOnceWith(
      expect.any(Number),
      "C:\\in",
      [RECORD],
      [METADATA],
      []
    );
  });
});
