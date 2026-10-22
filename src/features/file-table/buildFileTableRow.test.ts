import { describe, expect, it } from "vitest";
import type { FileRecord, MetadataResult } from "../../lib/types";
import { buildFileTableRow } from "./buildFileTableRow";

const RECORD = { id: "f1", relative_path: "a.jpg", scan_status: "discovered" } as FileRecord;
const row = (record: FileRecord, metadata?: MetadataResult, place?: string) =>
  buildFileTableRow(
    { record, metadata, movePlan: undefined, moveResult: undefined, planError: undefined },
    null,
    null,
    place
  );

describe("buildFileTableRow", () => {
  it("says what the date column is waiting for", () => {
    expect(row(RECORD).date.value).toBe("Reading...");
    expect(row(RECORD, { metadata_status: "no_date" } as MetadataResult).date.value).toBe(
      "No date"
    );
    expect(row({ ...RECORD, scan_status: "skipped_unsupported" }).date.value).toBe("-");
  });

  it("shows the place label, or an empty cell for a file with no GPS", () => {
    expect(row(RECORD, undefined, "Louvre, France").place.value).toBe("Louvre, France");
    expect(row(RECORD).place.value).toBe("");
  });
});
