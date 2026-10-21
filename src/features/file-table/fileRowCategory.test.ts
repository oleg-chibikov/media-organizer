import { describe, expect, it } from "vitest";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";
import { fileRowCategory } from "./fileRowCategory";
import type { FileRowState } from "./fileRowState";

const READY_STATE: FileRowState = {
  record: { id: "f1", scan_status: "discovered" } as FileRecord,
  metadata: { metadata_status: "ready", date_kind: "timezone_aware" } as MetadataResult,
  movePlan: { plan_status: "ready" } as MovePlan,
  moveResult: undefined,
  planError: undefined
};

const state = (overrides: Partial<FileRowState>) => ({ ...READY_STATE, ...overrides });

describe("fileRowCategory", () => {
  it("marks files that can't move as errors", () => {
    expect(
      fileRowCategory(state({ metadata: { metadata_status: "error" } as MetadataResult }))
    ).toBe("error");
    expect(fileRowCategory(state({ movePlan: undefined, planError: "Bad date" }))).toBe("error");
    expect(fileRowCategory(state({ moveResult: { result_status: "failed" } as MoveResult }))).toBe(
      "error"
    );
  });

  it("marks guessed dates and files left in place for review", () => {
    const fallback = { metadata_status: "ready", date_kind: "fallback" } as MetadataResult;
    expect(fileRowCategory(state({ metadata: fallback }))).toBe("review");
    expect(
      fileRowCategory(state({ movePlan: { plan_status: "blocked_no_date" } as MovePlan }))
    ).toBe("review");
  });

  it("leaves ready and moved files alone", () => {
    expect(fileRowCategory(READY_STATE)).toBe("other");
    expect(fileRowCategory(state({ moveResult: { result_status: "moved" } as MoveResult }))).toBe(
      "other"
    );
  });
});
