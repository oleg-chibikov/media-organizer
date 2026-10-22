import { describe, expect, it } from "vitest";
import { placePromptLabel } from "./placePromptLabel";
import { CONTINUED_TRIP, OUTING, TRIP } from "./testSuggestions";

describe("placePromptLabel", () => {
  it("names one trip with its files and days", () => {
    expect(placePromptLabel([TRIP])).toBe("Trip to United States: 4 files, 3 days. Review folders");
  });

  it("names one outing with its files", () => {
    expect(placePromptLabel([OUTING])).toBe("Royal Botanic Gardens: 2 files. Review folders");
  });

  it("names the placed trip a suggestion continues", () => {
    expect(placePromptLabel([CONTINUED_TRIP])).toBe(
      "Continues United States (Oct 20, 2025–Oct 27, 2025): 4 files, 3 days. Review folders"
    );
  });

  it("counts several suggestions", () => {
    expect(placePromptLabel([TRIP, OUTING, { ...OUTING, id: "o2" }])).toBe(
      "3 place folders suggested. Review"
    );
  });
});
