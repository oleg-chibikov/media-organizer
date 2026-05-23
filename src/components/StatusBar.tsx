interface StatusBarProps {
  discovered: number;
  skipped: number;
  errors: number;
  planErrors: number;
  isPlanning: boolean;
  planCompleted: boolean;
  completed: boolean;
  cancelled: boolean;
}

export function StatusBar({
  discovered,
  skipped,
  errors,
  planErrors,
  isPlanning,
  planCompleted,
  completed,
  cancelled
}: StatusBarProps) {
  const statusLabel = cancelled
    ? "Scan cancelled"
    : isPlanning
      ? "Planning destinations"
      : planCompleted
        ? "Plan completed"
        : completed
          ? "Scan completed"
          : "Idle";

  return (
    <footer className="flex items-center gap-4 border-t border-slate-800 bg-slate-900 px-3 py-2 text-xs text-slate-300">
      <span>Discovered: {discovered}</span>
      <span>Skipped: {skipped}</span>
      <span>Errors: {errors}</span>
      <span>Plan Errors: {planErrors}</span>
      <span className="ml-auto">
        {statusLabel}
      </span>
    </footer>
  );
}
