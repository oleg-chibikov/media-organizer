import { describe, expect, it } from "vitest";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";
import { fileNotesLabel } from "./fileNotesLabel";
import type { FileRowState } from "./fileRowState";
import { fileStatus } from "./fileStatus";

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

describe("fileStatus", () => {
  it("follows the file from planning to the move result", () => {
    const busy = (label: string) => ({ label, busy: true });
    expect(fileStatus(state({ metadata: undefined }), null)).toEqual(busy("Reading metadata"));
    expect(fileStatus(state(), null)).toEqual(busy("Planning"));
    expect(fileStatus(state(), "f1")).toEqual(busy("Moving"));
    const suffixed = { plan_status: "ready", conflict_status: "suffix_applied" } as MovePlan;
    expect(fileStatus(state({ movePlan: suffixed }), null).label).toBe("Ready (renamed)");
    const named = { plan_status: "already_named" } as MovePlan;
    expect(fileStatus(state({ movePlan: named }), null).label).toBe("Already named");
    const copied = { result_status: "copied" } as MoveResult;
    expect(fileStatus(state({ moveResult: copied }), null).label).toBe("Copied, original kept");
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
