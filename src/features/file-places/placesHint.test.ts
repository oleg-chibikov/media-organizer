import { describe, expect, it } from "vitest";
import { placesHint } from "./placesHint";

describe("placesHint", () => {
  it("counts the placed files, or says why none have a place", () => {
    expect(placesHint(null, 0, true)).toContain("Finding places...");
    expect(placesHint({ kind: "found" }, 3, false)).toContain("3 files have a place.");
    expect(placesHint({ kind: "found" }, 1, false)).toContain("1 file has a place.");
    expect(placesHint({ kind: "no_gps" }, 0, false)).toContain("No GPS in these files.");
    expect(placesHint(null, 0, false)).toContain("Locations weren't checked.");
  });
});
