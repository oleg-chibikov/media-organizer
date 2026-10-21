import { describe, expect, it } from "vitest";
import { IDLE_MOVE_EXECUTION } from "../moves/moveExecutionState";
import { EMPTY_SCAN_PROGRESS } from "../scan/scanProgress";
import { executionActionLabel, pipelineStatusLabel } from "./statusBarLabels";

const SCANNED = { ...EMPTY_SCAN_PROGRESS, discovered: 2, completed: true };
const PLANNED = { isPlanning: false, planCompleted: true, planErrors: 0 };

describe("pipelineStatusLabel", () => {
  it("moves from planning to ready to executing", () => {
    expect(pipelineStatusLabel(EMPTY_SCAN_PROGRESS, PLANNED, IDLE_MOVE_EXECUTION)).toBe("No files");
    const planning = { ...PLANNED, planCompleted: false };
    expect(pipelineStatusLabel(SCANNED, planning, IDLE_MOVE_EXECUTION)).toBe(
      "Planning destinations"
    );
    expect(pipelineStatusLabel(SCANNED, PLANNED, IDLE_MOVE_EXECUTION)).toBe("Destinations ready");
    const executing = { ...IDLE_MOVE_EXECUTION, executing: true };
    expect(pipelineStatusLabel(SCANNED, PLANNED, executing)).toBe("Executing moves");
  });
});

describe("executionActionLabel", () => {
  it("offers confirm, then pause and resume", () => {
    expect(executionActionLabel(IDLE_MOVE_EXECUTION)).toBe("Confirm & Execute");
    const running = { ...IDLE_MOVE_EXECUTION, confirmed: true };
    expect(executionActionLabel(running)).toBe("Pause Dispatch");
    expect(executionActionLabel({ ...running, paused: true })).toBe("Resume Dispatch");
  });
});
