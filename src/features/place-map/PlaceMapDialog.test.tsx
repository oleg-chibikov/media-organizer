import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PlaceMapDialog } from "./PlaceMapDialog";

vi.mock("./LeafletMap", () => ({
  LeafletMap: ({ label }: { label: string }) => <div role="img" aria-label={`Map of ${label}`} />
}));

afterEach(() => {
  cleanup();
});

const point = { latitude: -33.856784, longitude: 151.215297 };

describe("PlaceMapDialog", () => {
  it("shows the place on a map with its coordinates", async () => {
    render(<PlaceMapDialog target={{ point, label: "Sydney" }} onClose={vi.fn()} />);

    expect(screen.getByRole("heading", { name: "Sydney" })).toBeInTheDocument();
    expect(await screen.findByRole("img", { name: "Map of Sydney" })).toBeInTheDocument();
    expect(screen.getByText("-33.85678, 151.21530")).toBeInTheDocument();
  });

  it("falls back to coordinates before the place has a name", () => {
    render(<PlaceMapDialog target={{ point, label: "" }} onClose={vi.fn()} />);

    expect(screen.getByRole("heading", { name: "-33.85678, 151.21530" })).toBeInTheDocument();
  });

  it("closes from the Close button", () => {
    const onClose = vi.fn();
    render(<PlaceMapDialog target={{ point, label: "Sydney" }} onClose={onClose} />);

    fireEvent.click(screen.getByRole("button", { name: "Close" }));

    expect(onClose).toHaveBeenCalledOnce();
  });
});
