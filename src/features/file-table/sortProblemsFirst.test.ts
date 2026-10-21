import { describe, expect, it } from "vitest";
import type { FileRecord, MetadataResult, MovePlan } from "../../lib/types";
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

describe("sortProblemsFirst", () => {
  it("puts errors, then files to review, before the rest in their own order", () => {
    const rows = [
      state("a", "ready"),
      state("b", "blocked_no_date"),
      state("c", "ready"),
      state("d", "ready", "error"),
      state("e", "blocked_no_date")
    ];

    expect(sortProblemsFirst(rows).map((row) => row.record.id)).toEqual(["d", "b", "e", "a", "c"]);
  });

  it("returns the same list when nothing needs attention", () => {
    const rows = [state("a", "ready")];
    expect(sortProblemsFirst(rows)).toBe(rows);
  });
});
