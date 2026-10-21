import { describe, expect, it } from "vitest";
import { RECENT_RUN_COLUMNS, RECENT_RUN_DEFAULT_SIZING } from "./recentRunRows";

// The History pane is 56rem wide with 0.5rem padding on each side.
const PANE_CONTENT_WIDTH = 56 * 16 - 16;

describe("RECENT_RUN_COLUMNS", () => {
  it("fits every column up to Actions inside the History pane", () => {
    const keys = RECENT_RUN_COLUMNS.map((column) => column.key);
    const throughActions = keys.slice(0, keys.indexOf("actions") + 1);
    const width = throughActions.reduce((sum, key) => sum + RECENT_RUN_DEFAULT_SIZING[key], 0);

    expect(width).toBeLessThanOrEqual(PANE_CONTENT_WIDTH);
  });
});
