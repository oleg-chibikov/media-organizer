import { describe, expect, it } from "vitest";
import type { MoveConfirmationSummary } from "../../lib/types";
import { moveProgressCount } from "./moveProgressCount";

const summary = (skipped: number) => ({ skipped_operations: skipped }) as MoveConfirmationSummary;

describe("moveProgressCount", () => {
  it("counts every found file that can still move, checked or not", () => {
    const counts = { moved: 10, copied: 1, skipped: 1, failed: 1 };
    expect(moveProgressCount(300, 2, summary(5), counts)).toBe("13/293");
  });

  it("doesn't show fewer files in total than already have a result", () => {
    const counts = { moved: 4, copied: 0, skipped: 0, failed: 0 };
    expect(moveProgressCount(3, 0, summary(0), counts)).toBe("4/4");
  });
});
