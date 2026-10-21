import { describe, expect, it } from "vitest";
import { isProcessedPath } from "./isProcessedPath";

describe("isProcessedPath", () => {
  it("matches a Processed segment with either separator", () => {
    expect(isProcessedPath("C:\\Photos\\Processed\\2026")).toBe(true);
    expect(isProcessedPath("/home/me/processed")).toBe(true);
  });

  it("ignores names that only contain the word", () => {
    expect(isProcessedPath("C:\\Photos\\Unprocessed")).toBe(false);
    expect(isProcessedPath("/home/me/processed-later")).toBe(false);
  });
});
