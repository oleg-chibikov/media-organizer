import { useId } from "react";

interface InfoHintProps {
  text: string;
}

// An "i" badge that shows `text` on hover and on keyboard focus.
export function InfoHint({ text }: InfoHintProps) {
  const tipId = useId();
  return (
    <span className="group relative inline-flex">
      <span
        role="img"
        tabIndex={0}
        aria-label="More info"
        aria-describedby={tipId}
        className="inline-flex h-4 w-4 cursor-help items-center justify-center rounded-full border border-slate-400 text-[10px] font-semibold leading-none text-slate-300 hover:border-sky-400 hover:text-sky-300 focus-visible:outline-2 focus-visible:outline-sky-400"
      >
        i
      </span>
      <span
        id={tipId}
        role="tooltip"
        className="pointer-events-none invisible absolute bottom-full right-0 z-20 mb-2 w-72 rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-normal text-slate-200 shadow-lg group-focus-within:visible group-hover:visible"
      >
        {text}
      </span>
    </span>
  );
}
