import { describe, expect, it } from "vitest";
import { finishSummary } from "./finishSummary";

describe("finishSummary", () => {
  it("says what moved, what didn't and how to undo", () => {
    expect(finishSummary({ moved: 56, copied: 0, skipped: 0, failed: 1 })).toBe(
      "Done. Moved 56 files. 1 file didn't move. See Errors. You can undo this run in History."
    );
    expect(finishSummary({ moved: 1, copied: 2, skipped: 0, failed: 0 })).toBe(
      "Done. Moved 1 file. Copied 2 files and kept the originals. You can undo this run in History."
    );
  });
});
