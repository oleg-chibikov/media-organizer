import { describe, expect, it, vi } from "vitest";
import { confirmHistoryAction } from "./historyActionPrompt";

describe("confirmHistoryAction", () => {
  it.each(["undo", "redo", "delete"] as const)(
    "asks before %s and returns the answer",
    (action) => {
      const declined = vi.fn(() => false);
      const accepted = vi.fn(() => true);

      expect(confirmHistoryAction(action, declined)).toBe(false);
      expect(confirmHistoryAction(action, accepted)).toBe(true);
      expect(declined).toHaveBeenCalledWith(expect.stringMatching(/\?/));
    }
  );
});
