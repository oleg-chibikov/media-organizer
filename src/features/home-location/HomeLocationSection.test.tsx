import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { searchHomeCities } from "../../lib/api";
import type { HomeCandidate, HomeLocation } from "../../lib/types";
import { HomeLocationSection } from "./HomeLocationSection";

vi.mock("../../lib/api", () => ({ searchHomeCities: vi.fn() }));

const search = vi.mocked(searchHomeCities);
const SYDNEY: HomeLocation = {
  name: "Sydney",
  country: "Australia",
  latitude: -33.87,
  longitude: 151.21,
  source: "learned"
};
const CITIES: HomeCandidate[] = [
  { name: "Zuerich", region: "Zurich", country: "Switzerland", latitude: 47.37, longitude: 8.55 },
  { name: "Zuru", region: null, country: "Nigeria", latitude: 11.44, longitude: 5.23 }
];

function renderSection(home: HomeLocation | null = null) {
  const onChange = vi.fn();
  render(<HomeLocationSection home={home} disabled={false} onChange={onChange} />);
  return { onChange, input: screen.getByRole("combobox", { name: "Find a city" }) };
}

async function typeAndWait(input: HTMLElement, text: string) {
  fireEvent.change(input, { target: { value: text } });
  await act(async () => vi.advanceTimersByTimeAsync(200));
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => {
  cleanup();
  vi.useRealTimers();
  search.mockReset();
});

describe("HomeLocationSection", () => {
  it("shows the saved home and how it was set", () => {
    renderSection(SYDNEY);

    expect(screen.getByText("Home: Sydney, Australia (learned)")).toBeInTheDocument();
  });

  it("shows when home is not set and offers nothing to clear", () => {
    renderSection();

    expect(screen.getByText("Home: not set")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Clear home" })).toBeDisabled();
  });

  it("clears the saved home", () => {
    const { onChange } = renderSection(SYDNEY);

    fireEvent.click(screen.getByRole("button", { name: "Clear home" }));

    expect(onChange).toHaveBeenCalledWith(null);
  });

  it("searches once typing pauses", async () => {
    search.mockResolvedValue(CITIES);
    const { input } = renderSection();

    fireEvent.change(input, { target: { value: "z" } });
    act(() => vi.advanceTimersByTime(100));
    fireEvent.change(input, { target: { value: "zur" } });
    await act(async () => vi.advanceTimersByTimeAsync(200));

    expect(search).toHaveBeenCalledTimes(1);
    expect(search).toHaveBeenCalledWith("zur");
    const options = screen.getAllByRole("option");
    expect(options.map((option) => option.textContent)).toEqual([
      "Zuerich, Zurich, Switzerland",
      "Zuru, Nigeria"
    ]);
    expect(input).toHaveAttribute("aria-expanded", "true");
  });

  it("picks a city with the arrow keys and Enter", async () => {
    search.mockResolvedValue(CITIES);
    const { input, onChange } = renderSection();
    await typeAndWait(input, "zur");

    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "ArrowUp" });

    const [first] = screen.getAllByRole("option");
    expect(first).toHaveAttribute("aria-selected", "true");
    expect(input).toHaveAttribute("aria-activedescendant", first.id);
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).toHaveBeenCalledWith({
      name: "Zuerich",
      country: "Switzerland",
      latitude: 47.37,
      longitude: 8.55,
      source: "manual"
    });
    expect(input).toHaveValue("");
    expect(screen.queryAllByRole("option")).toEqual([]);
  });

  it("picks a city with a click", async () => {
    search.mockResolvedValue(CITIES);
    const { input, onChange } = renderSection();
    await typeAndWait(input, "zur");

    fireEvent.click(screen.getByRole("option", { name: "Zuru, Nigeria" }));

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({ name: "Zuru" }));
  });

  it("hides old matches as soon as the text changes", async () => {
    search.mockResolvedValue(CITIES);
    const { input } = renderSection();
    await typeAndWait(input, "zur");

    fireEvent.change(input, { target: { value: "zuri" } });

    expect(screen.queryAllByRole("option")).toEqual([]);
    expect(screen.getByText("Searching…")).toBeInTheDocument();
  });

  it("says when no city matches", async () => {
    search.mockResolvedValue([]);
    const { input } = renderSection();

    await typeAndWait(input, "qqq");

    expect(screen.getByText("No cities found.")).toBeInTheDocument();
  });
});
