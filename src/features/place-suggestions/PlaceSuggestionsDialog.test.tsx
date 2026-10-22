import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { MetadataResult } from "../../lib/types";
import { PlaceSuggestionsDialog } from "./PlaceSuggestionsDialog";
import { OUTING, TRIP, TRIP_DATES } from "./testSuggestions";

const metadataByFileId = Object.fromEntries(
  Object.entries(TRIP_DATES).map(([id, date]) => [id, { chosen_date: date } as MetadataResult])
);

function renderDialog() {
  const onApply = vi.fn();
  render(
    <PlaceSuggestionsDialog
      suggestions={[TRIP, OUTING]}
      choices={{}}
      metadataByFileId={metadataByFileId}
      onApply={onApply}
      onCancel={vi.fn()}
    />
  );
  return onApply;
}

afterEach(cleanup);

describe("PlaceSuggestionsDialog", () => {
  it("applies edited names and unchecked suggestions", () => {
    const onApply = renderDialog();
    fireEvent.change(screen.getByLabelText("Folder for Yosemite"), {
      target: { value: "Yosemite NP" }
    });
    const outing = screen.getByRole("region", { name: "Royal Botanic Gardens" });
    fireEvent.click(within(outing).getByRole("checkbox", { name: "Use place folders" }));

    const preview = screen.getByRole("list", { name: "Folder preview" });
    expect(within(preview).getByText("2025/United States/Yosemite NP/10-28 (1)")).toBeVisible();
    expect(within(outing).getByText("These files go to date folders.")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(onApply).toHaveBeenCalledWith({
      [TRIP.id]: {
        use: true,
        title: "United States",
        placeNames: ["Yosemite NP", "San Francisco"]
      },
      [OUTING.id]: { use: false, title: OUTING.title, placeNames: [OUTING.title] }
    });
  });

  it("disables Apply while a name is invalid", () => {
    renderDialog();
    const title = screen.getByLabelText("Trip folder");
    fireEvent.change(title, { target: { value: "US/A" } });

    expect(title).toHaveAccessibleDescription('Remove / \\ : * ? " < > |');
    expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();

    fireEvent.change(title, { target: { value: "USA" } });
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled();
  });
});
