import { describe, expect, it } from "vitest";
import { folderNameError } from "./folderNameError";

describe("folderNameError", () => {
  it("accepts a plain name", () => {
    expect(folderNameError(" Yosemite ")).toBeNull();
  });

  it.each(["", "   "])("asks for a name when it is %j", (name) => {
    expect(folderNameError(name)).toBe("Enter a folder name.");
  });

  it.each(["a/b", "a\\b", "a:b", "a*", "a?", 'a"', "a<", "a>", "a|b"])("rejects %j", (name) => {
    expect(folderNameError(name)).toBe('Remove / \\ : * ? " < > |');
  });

  it("rejects a name made of dots", () => {
    expect(folderNameError("..")).toBe("Use a name with letters or digits.");
  });
});
