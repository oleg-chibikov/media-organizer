import { describe, expect, it } from "vitest";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";
import { fileNotesLabel } from "./fileNotesLabel";
import type { FileRowState } from "./fileRowState";
import { fileStatusLabel } from "./fileStatusLabel";

const RECORD = { id: "f1", scan_status: "discovered" } as FileRecord;
const READY = {
  metadata_status: "ready",
  chosen_date_source: "DateTimeOriginal"
} as MetadataResult;

function state(overrides: Partial<FileRowState> = {}): FileRowState {
  return {
    record: RECORD,
    metadata: READY,
    movePlan: undefined,
    moveResult: undefined,
    planError: undefined,
    ...overrides
  };
}

describe("fileStatusLabel", () => {
  it("follows the file from planning to the move result", () => {
    expect(fileStatusLabel(state({ metadata: undefined }), null)).toBe("Reading date");
    expect(fileStatusLabel(state(), null)).toBe("Planning");
    expect(fileStatusLabel(state(), "f1")).toBe("Moving");
    const suffixed = { plan_status: "ready", conflict_status: "suffix_applied" } as MovePlan;
    expect(fileStatusLabel(state({ movePlan: suffixed }), null)).toBe("Ready (renamed)");
    const named = { plan_status: "already_named" } as MovePlan;
    expect(fileStatusLabel(state({ movePlan: named }), null)).toBe("Already named");
    const copied = { result_status: "copied" } as MoveResult;
    expect(fileStatusLabel(state({ moveResult: copied }), null)).toBe("Copied, original kept");
  });
});

describe("fileNotesLabel", () => {
  it("prefers move errors, then explains the date source", () => {
    expect(fileNotesLabel(state({ moveResult: { error: "Disk full" } as MoveResult }))).toBe(
      "Disk full"
    );
    expect(fileNotesLabel(state())).toBe("Photo capture time");
  });
});
