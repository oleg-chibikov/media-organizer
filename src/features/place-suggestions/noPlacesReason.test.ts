import { describe, expect, it } from "vitest";
import { noPlacesReason } from "./noPlacesReason";

describe("noPlacesReason", () => {
  it("explains each status the backend reports", () => {
    expect(noPlacesReason({ kind: "no_gps" })).toBe("No GPS in these files");
    expect(noPlacesReason({ kind: "home_unknown" })).toBe("Set home to find trips");
    expect(noPlacesReason({ kind: "all_near_home", home: "Sydney" })).toBe(
      "All files are near home (Sydney)"
    );
    expect(noPlacesReason({ kind: "too_few_geotagged" })).toBe("Too few geotagged files");
    expect(noPlacesReason(null)).toBe("Locations weren't checked");
  });
});
