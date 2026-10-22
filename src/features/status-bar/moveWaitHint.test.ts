import { describe, expect, it } from "vitest";
import { moveWaitHint } from "./moveWaitHint";

describe("moveWaitHint", () => {
  it("says files wait for every date only when grouping by place", () => {
    expect(moveWaitHint(true)).toContain("once every date is read");
    expect(moveWaitHint(false)).toContain("as soon as its date is read");
  });
});
