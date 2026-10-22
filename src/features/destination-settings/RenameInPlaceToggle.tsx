import { ToolbarCheckbox } from "../../components/design-system/ToolbarCheckbox";

interface RenameInPlaceToggleProps {
  checked: boolean;
  disabled: boolean;
  onChange: (nextValue: boolean) => void;
}

// A quick override next to the settings button: files keep their folder and only get a new name.
export function RenameInPlaceToggle(props: RenameInPlaceToggleProps) {
  return (
    <ToolbarCheckbox
      label="Rename only"
      hint="Files stay in their folders. Only the names change."
      {...props}
    />
  );
}
