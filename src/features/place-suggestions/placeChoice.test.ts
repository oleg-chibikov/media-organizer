import { describe, expect, it } from "vitest";
import { acceptedSuggestions, freshChoice, isChoiceValid, withChoiceNames } from "./placeChoice";
import { CONTINUED_TRIP, LIFTED_TRIP, OUTING, TRIP } from "./testSuggestions";

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

  it("keeps the country as the folder of a trip abroad", () => {
    const abroad = { ...TRIP, country: "United States" };
    const choice = { ...freshChoice(abroad), title: "a/b" };

    expect(isChoiceValid(abroad, choice)).toBe(true);
    expect(withChoiceNames(abroad, choice).title).toBe(TRIP.title);
  });

  it("keeps a country wide continuation for a renamed outing abroad", () => {
    const outing = {
      ...OUTING,
      country: "USA",
      title: "Zoo",
      places: [{ ...OUTING.places[0]!, name: "Zoo" }],
      continues: LIFTED_TRIP.continues
    };
    const renamed = withChoiceNames(outing, { ...freshChoice(outing), placeNames: ["City Zoo"] });

    expect(renamed.continues).toEqual(LIFTED_TRIP.continues);
  });
});
