import { describe, expect, it } from "vitest";
import { acceptedSuggestions, freshChoice, isChoiceValid, withChoiceNames } from "./placeChoice";
import { CONTINUED_TRIP, OUTING, TRIP } from "./testSuggestions";

describe("placeChoice", () => {
  it("accepts used suggestions with trimmed edited names", () => {
    const choices = {
      [TRIP.id]: { use: true, title: " USA ", placeNames: ["Yosemite NP", "SF"] },
      [OUTING.id]: { ...freshChoice(OUTING), use: false }
    };
    const [trip, ...rest] = acceptedSuggestions([TRIP, OUTING], choices);
    expect(rest).toEqual([]);
    expect(trip?.title).toBe("USA");
    expect(trip?.places.map((place) => place.name)).toEqual(["Yosemite NP", "SF"]);
  });

  it("checks names only while the suggestion is used", () => {
    const invalid = { ...freshChoice(TRIP), title: "a/b" };
    expect(isChoiceValid(TRIP, invalid)).toBe(false);
    expect(isChoiceValid(TRIP, { ...invalid, use: false })).toBe(true);
  });

  it("drops the continuation of a renamed trip and keeps it for a renamed place", () => {
    const fresh = freshChoice(CONTINUED_TRIP);
    const renamedTrip = withChoiceNames(CONTINUED_TRIP, { ...fresh, title: "USA" });
    const renamedPlace = withChoiceNames(CONTINUED_TRIP, { ...fresh, placeNames: ["NP", "SF"] });

    expect(renamedTrip.continues).toBeNull();
    expect(renamedPlace.continues).toEqual(CONTINUED_TRIP.continues);
  });
});
