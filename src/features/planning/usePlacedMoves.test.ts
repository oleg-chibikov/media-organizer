import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { createMovePlan, planContinuedMoves, type PlanListeners } from "../../lib/api";
import type { PlaceSuggestion, PlacedMove } from "../../lib/types";
import { METADATA, PLAN, RECORD } from "../../test/pipelineFixtures";
import { useMovePlanning } from "./useMovePlanning";

const listeners = vi.hoisted(() => ({ plan: null as PlanListeners | null }));

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  createMovePlan: vi.fn(async () => undefined),
  planContinuedMoves: vi.fn(),
  attachPlanListeners: vi.fn(async (handlers: PlanListeners) => ((listeners.plan = handlers), []))
}));

const PLACED_RECORD = { ...RECORD, id: "C:\\Root\\2025\\USA\\Vancouver\\c.jpg" };
const PLACED: PlacedMove = {
  record: PLACED_RECORD,
  metadata: { ...METADATA, file_id: PLACED_RECORD.id },
  plan: { ...PLAN, file_id: PLACED_RECORD.id }
};
const CONTINUED = {
  id: "t1",
  kind: "trip",
  title: "USA",
  places: [{ name: "Vancouver", file_ids: ["f1"] }],
  root_file_ids: [],
  continues: { title: "USA", year: 2025, start: "", end: "", places: [] }
} as unknown as PlaceSuggestion;

async function renderPlanning() {
  const onPlacedFiles = vi.fn();
  const hook = renderHook(() =>
    useMovePlanning({ selectedFolder: "C:\\in", reportError: vi.fn(), onPlacedFiles })
  );
  await waitFor(() => expect(listeners.plan).not.toBeNull());
  return { ...hook, onPlacedFiles };
}

describe("useMovePlanning placed files", () => {
  it("adds the placed files a continued trip moves and keeps them out of date planning", async () => {
    vi.mocked(planContinuedMoves).mockResolvedValueOnce([PLACED]);
    const { result, onPlacedFiles } = await renderPlanning();

    await act(async () => result.current.applyPlaceSuggestions([CONTINUED], [RECORD], [METADATA]));

    expect(planContinuedMoves).toHaveBeenCalledWith("C:\\in", [CONTINUED], [METADATA]);
    expect(onPlacedFiles).toHaveBeenCalledWith([PLACED], []);
    expect(result.current.movePlansByFileId[PLACED_RECORD.id]).toEqual(PLACED.plan);

    vi.mocked(createMovePlan).mockClear();
    act(() => result.current.requestPlans([PLACED_RECORD], [PLACED.metadata]));
    expect(createMovePlan).not.toHaveBeenCalled();
  });

  it("drops the placed files when the continued trip is no longer applied", async () => {
    vi.mocked(planContinuedMoves).mockResolvedValueOnce([PLACED]);
    const { result, onPlacedFiles } = await renderPlanning();
    await act(async () => result.current.applyPlaceSuggestions([CONTINUED], [RECORD], [METADATA]));
    vi.mocked(planContinuedMoves).mockClear();

    act(() => result.current.applyPlaceSuggestions([], [RECORD], [METADATA]));

    expect(planContinuedMoves).not.toHaveBeenCalled();
    expect(onPlacedFiles).toHaveBeenLastCalledWith([], [PLACED_RECORD.id]);
    expect(result.current.movePlansByFileId[PLACED_RECORD.id]).toBeUndefined();
  });

  it("drops placed files that arrive after a reset", async () => {
    let answer: (moves: PlacedMove[]) => void = () => undefined;
    vi.mocked(planContinuedMoves).mockReturnValueOnce(new Promise((resolve) => (answer = resolve)));
    const { result, onPlacedFiles } = await renderPlanning();
    act(() => result.current.applyPlaceSuggestions([CONTINUED], [RECORD], [METADATA]));

    act(() => result.current.resetPlanning());
    act(() => result.current.requestPlans([RECORD], [METADATA]));
    await act(async () => answer([PLACED]));

    expect(onPlacedFiles).not.toHaveBeenCalledWith([PLACED], []);
    expect(result.current.activePlanRequests).toBe(1);
  });
});
