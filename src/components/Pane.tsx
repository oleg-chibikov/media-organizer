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

  const close = useCallback(() => setOpen(false), []);
  const toggle = useCallback(() => setOpen((previous) => !previous), []);

  const handleClickOutside = useCallback((event: MouseEvent) => {
    if (paneRef.current && !paneRef.current.contains(event.target as Node)) {
      setOpen(false);
    }
  }, []);

  useEffect(() => {
    if (open) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open, handleClickOutside]);

  return (
    <div ref={paneRef} className={`relative ${className ?? ""}`.trim()}>
      {renderTrigger({ open, toggle, close })}
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
