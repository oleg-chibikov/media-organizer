import { describe, expect, it } from "vitest";
import { placesHint } from "./placesHint";
import { TRIP } from "./testSuggestions";

describe("placesHint", () => {
  it("names what detection found, or why it found no folders", () => {
    expect(placesHint(null, [], true)).toContain("Finding places...");
    expect(placesHint({ kind: "found" }, [TRIP], false)).toContain("Found 1 place:");
    expect(placesHint({ kind: "no_gps" }, [], false)).toContain("No GPS in these files.");
  });
});
