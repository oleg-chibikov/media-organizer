import { describe, expect, it } from "vitest";
import { FileIdMap } from "./fileIdMap";

const fileIdOf = (value: { id: string }) => value.id;

describe("FileIdMap", () => {
  it("returns a new handle with the added and removed values", () => {
    const empty = FileIdMap.empty<{ id: string }>();
    const added = empty.with([{ id: "a" }, { id: "b" }], fileIdOf);
    expect(added).not.toBe(empty);
    expect(added.size).toBe(2);

    const removed = added.without(["a"]);
    expect(removed).not.toBe(added);
    expect(removed.has("a")).toBe(false);
    expect(removed.get("b")).toEqual({ id: "b" });
  });

  it("keeps the handle when nothing changes, so React skips the render", () => {
    const map = FileIdMap.from({ a: { id: "a" } });
    expect(map.with([], fileIdOf)).toBe(map);
    expect(map.without(["missing"])).toBe(map);
  });
});
