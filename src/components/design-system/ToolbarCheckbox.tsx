interface ToolbarCheckboxProps {
  label: string;
  hint: string;
  checked: boolean;
  disabled: boolean;
  onChange: (nextValue: boolean) => void;
}

export function ToolbarCheckbox({
  label,
  hint,
  checked,
  disabled,
  onChange
}: ToolbarCheckboxProps) {
  return (
    <label
      className="flex cursor-pointer items-center gap-1.5 text-sm text-slate-200 has-disabled:cursor-not-allowed has-disabled:text-slate-500"
      title={hint}
    >
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      {label}
    </label>
  );
}
