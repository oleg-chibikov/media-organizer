import { describe, expect, it } from "vitest";
import type { DestinationSettings } from "../../lib/types";
import { effectiveDestinationMode } from "./effectiveDestinationMode";

const CUSTOM = {
  root_media_folder: "D:\\Media",
  force_processed_folder: false
} as DestinationSettings;

describe("effectiveDestinationMode", () => {
  it("uses the custom root only when set and not overridden", () => {
    expect(effectiveDestinationMode(CUSTOM)).toBe("Custom root active");
    expect(effectiveDestinationMode(null)).toBe("Processed active");
    expect(effectiveDestinationMode({ ...CUSTOM, force_processed_folder: true })).toBe(
      "Processed active"
    );
  });
});
