import { describe, expect, it } from "vitest";
import type { HistoryIndexEntry } from "../../lib/types";
import { resolveSelectedRunId } from "./resolveSelectedRunId";

const RUNS = [{ run_id: "a" }, { run_id: "b" }] as HistoryIndexEntry[];

describe("resolveSelectedRunId", () => {
  it("prefers the requested run", () => {
    expect(resolveSelectedRunId("a", "b", RUNS)).toBe("b");
  });

  it("keeps a listed selection and drops a removed one", () => {
    expect(resolveSelectedRunId("a", null, RUNS)).toBe("a");
    expect(resolveSelectedRunId("gone", null, RUNS)).toBeNull();
  });

  it("ignores a preferred run that is no longer listed", () => {
    expect(resolveSelectedRunId("a", "gone", RUNS)).toBe("a");
    expect(resolveSelectedRunId(null, "gone", RUNS)).toBeNull();
  });
});
