import { describe, expect, it } from "vitest";
import { relativeDestinationPath } from "./relativeDestinationPath";

describe("relativeDestinationPath", () => {
  it("strips the root media folder case-insensitively", () => {
    expect(relativeDestinationPath("D:\\Media\\2026\\a.jpg", "d:\\media\\")).toBe("2026/a.jpg");
  });

  it("keeps placeholders and paths outside the root", () => {
    expect(relativeDestinationPath("planning...", "D:\\Media")).toBe("planning...");
    expect(relativeDestinationPath("E:\\Other\\a.jpg", "D:\\Media")).toBe("E:\\Other\\a.jpg");
  });
});
