import { describe, expect, it } from "vitest";
import { placePromptLabel } from "./placePromptLabel";
import { OUTING, TRIP } from "./testSuggestions";

describe("placePromptLabel", () => {
  it("names one trip with its files and days", () => {
    expect(placePromptLabel([TRIP])).toBe("Trip to United States: 4 files, 3 days. Review folders");
  });

  it("names one outing with its files", () => {
    expect(placePromptLabel([OUTING])).toBe("Royal Botanic Gardens: 2 files. Review folders");
  });

  it("counts several suggestions", () => {
    expect(placePromptLabel([TRIP, OUTING, { ...OUTING, id: "o2" }])).toBe(
      "3 place folders suggested. Review"
    );
  });
});
