import { describe, expect, it } from "vitest";
import { IDLE_MOVE_EXECUTION } from "../moves";
import { EMPTY_SCAN_PROGRESS } from "../scan";
import { executionActionLabel, pipelineStatusLabel } from "./statusBarLabels";

const SCANNED = { ...EMPTY_SCAN_PROGRESS, discovered: 2, completed: true };
const PLANNED = { isPlanning: false, planCompleted: true, planErrors: 0 };

describe("pipelineStatusLabel", () => {
  it("moves from planning to ready to executing", () => {
    expect(pipelineStatusLabel(EMPTY_SCAN_PROGRESS, PLANNED, IDLE_MOVE_EXECUTION, 1)).toBe(
      "No files"
    );
    const planning = { ...PLANNED, planCompleted: false };
    expect(pipelineStatusLabel(SCANNED, planning, IDLE_MOVE_EXECUTION, 1)).toBe("Checking files");
    expect(pipelineStatusLabel(SCANNED, PLANNED, IDLE_MOVE_EXECUTION, 1)).toBe("Ready to move");
    const executing = { ...IDLE_MOVE_EXECUTION, executing: true };
    expect(pipelineStatusLabel(SCANNED, PLANNED, executing, 1)).toBe("Moving files");
  });

  it("says nothing is left to move when every file is already in place", () => {
    expect(pipelineStatusLabel(SCANNED, PLANNED, IDLE_MOVE_EXECUTION, 0)).toBe("Nothing to move");
  });

  it("says the folder is still scanning before the scan finishes", () => {
    const scanning = { ...SCANNED, completed: false };
    expect(pipelineStatusLabel(scanning, PLANNED, IDLE_MOVE_EXECUTION, 1)).toBe("Scanning");
  });
});

describe("executionActionLabel", () => {
  it("offers confirm, then pause and resume", () => {
    expect(executionActionLabel(IDLE_MOVE_EXECUTION, 1)).toBe("Move Files...");
    const running = { ...IDLE_MOVE_EXECUTION, confirmed: true };
    expect(executionActionLabel(running)).toBe("Pause");
    expect(executionActionLabel({ ...running, paused: true })).toBe("Resume");
  });
});
