import { describe, expect, it } from "vitest";
import type { FileRecord, MetadataResult } from "../../lib/types";
import { createFileRowCache } from "./fileRowCache";

const record = (id: string) =>
  ({ id, kind: "photo", scan_status: "discovered", relative_path: `${id}.jpg` }) as FileRecord;
const metadata = (id: string) =>
  ({ file_id: id, metadata_status: "ready", chosen_date: "2024-01-01" }) as MetadataResult;
const RECORDS = [record("a"), record("b")];
const NO_INPUTS = {
  metadataByFileId: {},
  movePlansByFileId: {},
  moveResultsByFileId: {},
  planErrorsByFileId: {}
};
const CONTEXT = {
  activeMoveFileId: null,
  destinationRoot: "/d",
  filePlaces: {},
  locatingPlaces: false
};

describe("createFileRowCache", () => {
  it("rebuilds only the rows whose inputs changed", () => {
    const cache = createFileRowCache();
    const first = cache.rows(cache.states(RECORDS, NO_INPUTS), CONTEXT);
    const inputs = { ...NO_INPUTS, metadataByFileId: { b: metadata("b") } };
    const second = cache.rows(cache.states(RECORDS, inputs), CONTEXT);

    expect(second[0]).toBe(first[0]);
    expect(second[1]).not.toBe(first[1]);
    expect(second[1].date.value).toBe("2024-01-01");
  });

  it("rebuilds the active row and every row when the destination changes", () => {
    const cache = createFileRowCache();
    const states = cache.states(RECORDS, NO_INPUTS);
    const first = cache.rows(states, CONTEXT);
    const active = cache.rows(states, { ...CONTEXT, activeMoveFileId: "b" });
    expect(active[0]).toBe(first[0]);
    expect(active[1]).not.toBe(first[1]);

    const moved = cache.rows(states, { ...CONTEXT, activeMoveFileId: "b", destinationRoot: "/e" });
    expect(moved[0]).not.toBe(active[0]);
  });

  it("forgets files that left the list", () => {
    const cache = createFileRowCache();
    const [, first] = cache.states(RECORDS, NO_INPUTS);
    cache.states([RECORDS[0]], NO_INPUTS);
    const [, again] = cache.states(RECORDS, NO_INPUTS);
    expect(again).not.toBe(first);
  });
});
