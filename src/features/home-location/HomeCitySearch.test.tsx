import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { searchHomeCities } from "../../lib/api";
import type { HomeCandidate } from "../../lib/types";
import { HomeLocationSection } from "./HomeLocationSection";

vi.mock("../../lib/api", () => ({ searchHomeCities: vi.fn() }));

const search = vi.mocked(searchHomeCities);
const CITIES: HomeCandidate[] = [
  { name: "Zuerich", region: "Zurich", country: "Switzerland", latitude: 47.37, longitude: 8.55 },
  { name: "Zuru", region: null, country: "Nigeria", latitude: 11.44, longitude: 5.23 }
];

function renderSection() {
  const onChange = vi.fn();
  render(
    <HomeLocationSection home={null} disabled={false} onChange={onChange} onLocate={vi.fn()} />
  );
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

describe("HomeCitySearch", () => {
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
