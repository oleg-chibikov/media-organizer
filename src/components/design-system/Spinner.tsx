// A small ring that spins only when the person allows motion. Pair it with visible text.
export function Spinner() {
  return (
    <span
      aria-hidden="true"
      className="inline-block h-3.5 w-3.5 shrink-0 rounded-full border-2 border-slate-600 border-t-sky-300 motion-safe:animate-spin"
    />
  );
}
