import { describe, expect, it } from "vitest";
import type { HistoryOperation, HistoryReplayProgressEvent } from "../../lib/types";
import { advanceHistoryAction, type ActiveHistoryAction } from "./historyActionProgress";

function progress(
  completed: number,
  status: HistoryOperation["status"]
): HistoryReplayProgressEvent {
  return {
    run_id: "run-1",
    total_operations: 2,
    completed_operations: completed,
    operation: { status } as HistoryOperation
  };
}

const ACTIVE: ActiveHistoryAction = { runId: "run-1", action: "undo", blockedOrFailed: 0 };

describe("advanceHistoryAction", () => {
  it("ignores progress for another action", () => {
    expect(advanceHistoryAction(ACTIVE, "redo", progress(1, "failed"))).toEqual({
      active: ACTIVE,
      failureMessage: null
    });
  });

  it("reports blocked or failed operations when the action finishes", () => {
    const first = advanceHistoryAction(ACTIVE, "undo", progress(1, "blocked"));
    expect(first.active?.blockedOrFailed).toBe(1);
    expect(advanceHistoryAction(first.active, "undo", progress(2, "succeeded"))).toEqual({
      active: null,
      failureMessage: "Undo could not be completed: 1 operation(s) were blocked or failed."
    });
  });
});
