import { ToolbarCheckbox } from "../../components/design-system/ToolbarCheckbox";

interface GroupByPlaceToggleProps {
  checked: boolean;
  renamesInPlace: boolean;
  disabled: boolean;
  // What detection found, or why it found no trips.
  placesHint: string;
  onChange: (nextValue: boolean) => void;
}

// Rename only keeps every file in its folder, so it shows the box off and locked.
export function GroupByPlaceToggle({
  checked,
  renamesInPlace,
  disabled,
  placesHint,
  onChange
}: GroupByPlaceToggleProps) {
  return (
    <ToolbarCheckbox
      label="Group by place"
      hint={renamesInPlace ? "Rename only keeps files in their folders." : placesHint}
      checked={checked && !renamesInPlace}
      disabled={disabled || renamesInPlace}
      onChange={onChange}
    />
  );
}
