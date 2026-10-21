import { describe, expect, it } from "vitest";
import type { FileRecord } from "../../lib/types";
import { sortFileRecords } from "./sortFileRecords";

function record(id: string, originalName: string, relativePath = originalName): FileRecord {
  return { id, original_name: originalName, relative_path: relativePath } as FileRecord;
}

describe("sortFileRecords", () => {
  it("sorts by name with numeric order, then by path", () => {
    const sorted = sortFileRecords([
      record("1", "IMG_10.jpg"),
      record("2", "img_2.jpg", "b/img_2.jpg"),
      record("3", "IMG_2.jpg", "a/IMG_2.jpg")
    ]);
    expect(sorted.map((row) => row.id)).toEqual(["3", "2", "1"]);
  });
});
