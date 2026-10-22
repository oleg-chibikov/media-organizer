import { describe, expect, it } from "vitest";
import type { FilePlace, FileRecord, MetadataResult, MovePlan } from "../../lib/types";
import { buildFileTableRow } from "./buildFileTableRow";

const RECORD = {
  id: "f1",
  relative_path: "a.jpg",
  kind: "photo",
  scan_status: "discovered"
} as FileRecord;
const row = (
  record: FileRecord,
  metadata?: MetadataResult,
  place?: FilePlace,
  locating = false,
  movePlan?: MovePlan
) =>
  buildFileTableRow(
    { record, metadata, movePlan, moveResult: undefined, planError: undefined },
    null,
    null,
    place,
    locating
  );

describe("buildFileTableRow", () => {
  it("says what the date column is waiting for", () => {
    expect(row(RECORD).date.value).toBe("Reading...");
    expect(row(RECORD, { metadata_status: "no_date" } as MetadataResult).date.value).toBe(
      "No date"
    );
    expect(row({ ...RECORD, scan_status: "skipped_unsupported" }).date.value).toBe("-");
  });

  it("shows the place and its country, or empty cells for a file with no place", () => {
    const louvre = row(RECORD, undefined, {
      place: "Louvre",
      country: "France",
      home: "away" as const
    });
    expect([louvre.country.value, louvre.place.value]).toEqual(["France", "Louvre"]);
    const sydney = row(RECORD, undefined, { place: "Sydney", country: "Australia", home: "city" });
    expect([sydney.country.value, sydney.place.value]).toEqual(["Australia", "Sydney"]);
    expect(row(RECORD).place.value).toBe("");
  });

  it("shows the planned folder, or a spinner while a dated file waits for its plan", () => {
    const ready = { metadata_status: "ready" } as MetadataResult;
    expect(row(RECORD, ready).destination).toMatchObject({ value: "Planning...", loading: true });
    const plan = { destination_path: "/Media/2024/3/15/a.jpg" } as MovePlan;
    expect(row(RECORD, ready, undefined, false, plan).destination.value).toBe(
      "/Media/2024/3/15/a.jpg"
    );
    expect(row(RECORD).destination.value).toBe("-");
  });

  it("shows a spinner in the place cell of photos and videos while locating", () => {
    expect(row(RECORD, undefined, undefined, true).place).toMatchObject({
      value: "Locating...",
      loading: true
    });
    const other = row({ ...RECORD, kind: "unsupported" } as FileRecord, undefined, undefined, true);
    expect(other.place.loading).toBeUndefined();
  });

  it("shows a place from a preview while locating goes on", () => {
    const osaka: FilePlace = { place: "Osaka", country: "Japan", home: "away" };
    expect(row(RECORD, undefined, osaka, true).place).toMatchObject({ value: "Osaka" });
  });
});
