import { Spinner } from "./Spinner";

// Muted text after a spinner, for a value that is still on its way. Adds the trailing dots.
export function LoadingText({ children }: { children: string }) {
  return (
    <span className="inline-flex items-center gap-1.5 text-slate-400">
      <Spinner />
      {`${children}...`}
    </span>
  );
}
