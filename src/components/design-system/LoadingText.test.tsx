import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { LoadingText } from "./LoadingText";

describe("LoadingText", () => {
  it("adds dots to the text and shows a spinner hidden from screen readers", () => {
    const { container } = render(<LoadingText>Planning</LoadingText>);

    expect(screen.getByText("Planning...")).toBeInTheDocument();
    expect(container.querySelector('[aria-hidden="true"]')).not.toBeNull();
  });
});
