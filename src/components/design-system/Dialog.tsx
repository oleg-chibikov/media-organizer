import { useEffect, useId, useRef, type ReactNode } from "react";

interface DialogProps {
  title: string;
  children: ReactNode;
  actions: ReactNode;
  wide?: boolean;
  onCancel: () => void;
}

// A modal shown while mounted. The browser traps focus and Escape calls onCancel.
export function Dialog({ title, children, actions, wide = false, onCancel }: DialogProps) {
  const ref = useRef<HTMLDialogElement>(null);
  const titleId = useId();

  useEffect(() => {
    const dialog = ref.current;
    if (dialog && !dialog.open) {
      dialog.showModal();
    }
    return () => dialog?.close();
  }, []);

  return (
    <dialog
      ref={ref}
      aria-labelledby={titleId}
      className={`m-auto ${wide ? "w-[44rem]" : "w-[32rem]"} max-w-[calc(100vw-2rem)] rounded-lg border border-slate-700 bg-slate-900 p-5 text-slate-100 shadow-2xl backdrop:bg-black/60`}
      onCancel={(event) => {
        event.preventDefault();
        onCancel();
      }}
    >
      <h2 id={titleId} className="text-lg font-semibold">
        {title}
      </h2>
      <div className="mt-3 text-sm text-slate-300">{children}</div>
      <div className="mt-5 flex justify-end gap-2">{actions}</div>
    </dialog>
  );
}
