interface StatusBarProps {
  discovered: number;
  skipped: number;
  errors: number;
  planErrors: number;
  isPlanning: boolean;
  planCompleted: boolean;
  completed: boolean;
  cancelled: boolean;
  isExecutingMoves: boolean;
  moveCompleted: boolean;
  moved: number;
  copied: number;
  moveSkipped: number;
  moveFailed: number;
}

export function StatusBar({
  discovered,
  skipped,
  errors,
  planErrors,
  isPlanning,
  planCompleted,
  completed,
  cancelled,
  isExecutingMoves,
  moveCompleted,
  moved,
  copied,
  moveSkipped,
  moveFailed
}: StatusBarProps) {
  const statusLabel = cancelled
    ? "Scan cancelled"
    : isExecutingMoves
      ? "Executing moves"
      : moveCompleted
        ? "Move completed"
        : completed && !planCompleted
          ? "Planning destinations"
          : isPlanning
            ? "Planning destinations"
            : completed && planCompleted
              ? "Destinations planned"
              : completed
                ? "Scan completed"
                : "Idle";

  return (
    <footer className="flex items-center gap-4 border-t border-slate-800 bg-slate-900 px-3 py-2 text-xs text-slate-300">
      <span>Discovered: {discovered}</span>
      <span>Skipped: {skipped}</span>
      <span>Errors: {errors}</span>
      <span>Plan Errors: {planErrors}</span>
      <span>Moved: {moved}</span>
      <span>Copied: {copied}</span>
      <span>Move Skipped: {moveSkipped}</span>
      <span>Move Failed: {moveFailed}</span>
      <span className="ml-auto">{statusLabel}</span>
    </footer>
  );
}
