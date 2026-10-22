import { useId } from "react";
import { folderNameError } from "./folderNameError";

interface FolderNameFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
}

export function FolderNameField({ label, value, onChange }: FolderNameFieldProps) {
  const id = useId();
  const errorId = `${id}-error`;
  const error = folderNameError(value);
  return (
    <div>
      <label htmlFor={id} className="block text-xs text-slate-400">
        {label}
      </label>
      <input
        id={id}
        value={value}
        aria-invalid={error !== null}
        aria-describedby={error ? errorId : undefined}
        className="mt-1 w-full rounded border border-slate-600 bg-slate-950 px-2 py-1 text-sm text-slate-100 focus-visible:outline-2 focus-visible:outline-sky-400 aria-invalid:border-rose-400"
        onChange={(event) => onChange(event.target.value)}
      />
      {error ? (
        <p id={errorId} className="mt-1 text-xs text-rose-300">
          {error}
        </p>
      ) : null}
    </div>
  );
}
