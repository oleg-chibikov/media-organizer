interface RenameInPlaceToggleProps {
  checked: boolean;
  disabled: boolean;
  onChange: (nextValue: boolean) => void;
}

// A quick override next to the settings button: files keep their folder and only get a new name.
export function RenameInPlaceToggle({ checked, disabled, onChange }: RenameInPlaceToggleProps) {
  return (
    <label
      className="flex cursor-pointer items-center gap-1.5 text-sm text-slate-200 has-disabled:cursor-not-allowed has-disabled:text-slate-500"
      title="Files stay in their folders. Only the names change."
    >
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      Rename only
    </label>
  );
}
