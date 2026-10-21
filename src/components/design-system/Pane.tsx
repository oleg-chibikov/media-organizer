import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";

type PaneAlign = "left" | "right";

interface PaneRenderProps {
  open: boolean;
  toggle: () => void;
  close: () => void;
}

interface PaneProps {
  align?: PaneAlign;
  className?: string;
  panelClassName?: string;
  children: ReactNode;
  renderTrigger: (props: PaneRenderProps) => ReactNode;
}

export function Pane({
  align = "left",
  className,
  panelClassName,
  children,
  renderTrigger
}: PaneProps) {
  const [open, setOpen] = useState(false);
  const paneRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLDivElement>(null);

  const close = useCallback(() => setOpen(false), []);
  const toggle = useCallback(() => setOpen((previous) => !previous), []);

  const handleClickOutside = useCallback((event: MouseEvent) => {
    if (paneRef.current && !paneRef.current.contains(event.target as Node)) {
      setOpen(false);
    }
  }, []);

  useEffect(() => {
    if (!open) {
      return;
    }
    // Escape closes the pane and puts focus back on the button that opened it.
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
        triggerRef.current?.querySelector<HTMLElement>("button")?.focus();
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, [open, handleClickOutside]);

  return (
    <div ref={paneRef} className={`relative ${className ?? ""}`.trim()}>
      <div ref={triggerRef} className="contents">
        {renderTrigger({ open, toggle, close })}
      </div>
      {open ? (
        <div
          className={`absolute ${align === "right" ? "right-0" : "left-0"} z-20 mt-1 rounded border border-slate-700 bg-slate-900 shadow-lg ${panelClassName ?? ""}`.trim()}
        >
          {children}
        </div>
      ) : null}
    </div>
  );
}
