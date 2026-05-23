interface StatusBarProps {
  discovered: number;
  skipped: number;
  errors: number;
  completed: boolean;
  cancelled: boolean;
}

export function StatusBar({
  discovered,
  skipped,
  errors,
  completed,
  cancelled
}: StatusBarProps) {
  return (
    <footer className="flex items-center gap-4 border-t border-slate-800 bg-slate-900 px-3 py-2 text-xs text-slate-300">
      <span>Discovered: {discovered}</span>
      <span>Skipped: {skipped}</span>
      <span>Errors: {errors}</span>
      <span className="ml-auto">
        {cancelled ? "Scan cancelled" : completed ? "Scan completed" : "Idle"}
      </span>
    </footer>
  );
}
