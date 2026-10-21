import { describe, expect, it } from "vitest";
import { historyDestinationRoot, toRelativeHistoryPath } from "./historyPaths";

describe("historyPaths", () => {
  it("falls back to the Processed folder for the destination root", () => {
    expect(historyDestinationRoot({ selected_folder: "C:\\In" })).toBe("C:\\In\\Processed");
    expect(
      historyDestinationRoot({ selected_folder: "C:\\In", destination_root_path: "D:\\M" })
    ).toBe("D:\\M");
  });

  it("makes paths relative to a root with either separator", () => {
    expect(toRelativeHistoryPath("C:/In/a/b.jpg", "c:\\in")).toBe("a/b.jpg");
    expect(toRelativeHistoryPath("C:\\In", "C:\\In")).toBe(".");
    expect(toRelativeHistoryPath("D:\\x.jpg", "C:\\In")).toBe("D:\\x.jpg");
  });
});
