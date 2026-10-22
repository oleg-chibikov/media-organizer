import { describe, expect, it } from "vitest";
import { FileIdMap } from "../../lib/fileIdMap";
import type { MoveResult } from "../../lib/types";
import { countMoveResults, hasMoveResults } from "./moveCounts";

function result(fileId: string, status: MoveResult["result_status"]): MoveResult {
  return {
    file_id: fileId,
    attempted_source_path: `C:\\in\\${fileId}.jpg`,
    attempted_destination_path: `C:\\in\\Processed\\${fileId}.jpg`,
    move_strategy: "rename",
    result_status: status,
    error: null
  };
}

describe("countMoveResults", () => {
  it("counts each file once by its latest status", () => {
    const counts = countMoveResults(
      FileIdMap.from({ a: result("a", "moved"), b: result("b", "failed") })
    );

    expect(counts).toEqual({ moved: 1, copied: 0, skipped: 0, failed: 1 });
    expect(hasMoveResults(counts)).toBe(true);
    expect(hasMoveResults(countMoveResults(FileIdMap.empty()))).toBe(false);
  });
});
