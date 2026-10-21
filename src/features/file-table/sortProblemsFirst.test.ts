import { describe, expect, it } from "vitest";
import type { FileRecord, MetadataResult, MovePlan } from "../../lib/types";
import { fileFilterRange } from "./fileFilter";
import type { FileRowState } from "./fileRowState";
import { sortProblemsFirst } from "./sortProblemsFirst";

function state(id: string, planStatus: MovePlan["plan_status"], metadataStatus = "ready") {
  return {
    record: { id, scan_status: "discovered" } as FileRecord,
    metadata: { metadata_status: metadataStatus } as MetadataResult,
    movePlan: { plan_status: planStatus } as MovePlan,
    moveResult: undefined,
    planError: undefined
  } as FileRowState;
}

const ROWS = [
  state("a", "ready"),
  state("b", "blocked_no_date"),
  state("c", "ready"),
  state("d", "ready", "error"),
  state("e", "blocked_no_date")
];

describe("sortProblemsFirst", () => {
  it("puts errors, then files to review, before the rest in their own order", () => {
    const sorted = sortProblemsFirst(ROWS);

    expect(sorted.states.map((row) => row.record.id)).toEqual(["d", "b", "e", "a", "c"]);
    expect(sorted).toMatchObject({ errorCount: 1, reviewCount: 2 });
  });

  it("returns the same list when nothing needs attention", () => {
    const rows = [state("a", "ready")];
    expect(sortProblemsFirst(rows).states).toBe(rows);
  });
});

describe("fileFilterRange", () => {
  it("picks each group out of the sorted rows", () => {
    const sorted = sortProblemsFirst(ROWS);

    expect(fileFilterRange("error", sorted)).toEqual([0, 1]);
    expect(fileFilterRange("review", sorted)).toEqual([1, 3]);
    expect(fileFilterRange("all", sorted)).toEqual([0, 5]);
  });
});
