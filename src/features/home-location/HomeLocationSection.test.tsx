import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { HomeLocation, HomeSource } from "../../lib/types";
import { HomeLocationSection } from "./HomeLocationSection";

vi.mock("../../lib/api", () => ({ searchHomeCities: vi.fn() }));

const sydney = (source: HomeSource): HomeLocation => ({
  name: "Sydney",
  country: "Australia",
  latitude: -33.87,
  longitude: 151.21,
  source
});

function renderSection(home: HomeLocation | null, onLocate = vi.fn().mockResolvedValue(undefined)) {
  const onChange = vi.fn();
  render(
    <HomeLocationSection home={home} disabled={false} onChange={onChange} onLocate={onLocate} />
  );
  return { onChange, onLocate };
}

afterEach(cleanup);

describe("HomeLocationSection", () => {
  it.each([
    ["learned", "learned from photos"],
    ["device", "from your location"],
    ["time_zone", "from time zone"],
    ["manual", "picked by you"]
  ] as const)("shows a %s home and where it came from", (source, label) => {
    renderSection(sydney(source));

    expect(screen.getByText(/^Home: Sydney, Australia/)).toHaveTextContent(
      `Home: Sydney, Australia · ${label}`
    );
  });

  it("shows when home is not set and offers nothing to clear", () => {
    renderSection(null);

    expect(screen.getByText("Home: not set")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Clear home" })).toBeDisabled();
  });

  it("clears the saved home", () => {
    const { onChange } = renderSection(sydney("learned"));

    fireEvent.click(screen.getByRole("button", { name: "Clear home" }));

    expect(onChange).toHaveBeenCalledWith(null);
  });

  it("shows progress while it reads the current location", async () => {
    let finish = () => {};
    const onLocate = vi.fn(() => new Promise<void>((resolve) => (finish = resolve)));
    renderSection(null, onLocate);

    fireEvent.click(screen.getByRole("button", { name: "Use current location" }));

    expect(onLocate).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("button", { name: "Locating..." })).toBeInTheDocument();
    await act(async () => finish());
    expect(screen.getByRole("button", { name: "Use current location" })).toBeInTheDocument();
  });
});
