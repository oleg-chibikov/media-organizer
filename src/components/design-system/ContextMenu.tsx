import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent } from "react";
import { createPortal } from "react-dom";

interface ContextMenuItem {
  label: string;
  onSelect: () => void;
}

interface ContextMenuProps {
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
}

// Portaled to the body so a transformed ancestor, like a virtual table row, can't shift it.
export function ContextMenu({ x, y, items, onClose }: ContextMenuProps) {
  const ref = useRef<HTMLUListElement>(null);
  const [position, setPosition] = useState({ left: x, top: y });

  useLayoutEffect(() => {
    const menu = ref.current;
    if (!menu) return;
    const { width, height } = menu.getBoundingClientRect();
    setPosition({
      left: Math.max(0, Math.min(x, window.innerWidth - width)),
      top: Math.max(0, Math.min(y, window.innerHeight - height))
    });
    menu.querySelector("button")?.focus();
  }, [x, y]);

  useEffect(() => {
    const previousFocus = document.activeElement as HTMLElement | null;
    const closeOutside = (event: Event) => {
      if (!ref.current?.contains(event.target as Node)) onClose();
    };
    document.addEventListener("pointerdown", closeOutside, true);
    window.addEventListener("blur", onClose);
    window.addEventListener("resize", onClose);
    return () => {
      document.removeEventListener("pointerdown", closeOutside, true);
      window.removeEventListener("blur", onClose);
      window.removeEventListener("resize", onClose);
      previousFocus?.focus();
    };
  }, [onClose]);

  const onKeyDown = (event: KeyboardEvent) => {
    const buttons = [...(ref.current?.querySelectorAll("button") ?? [])];
    const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
    const step = { ArrowDown: 1, ArrowUp: -1 }[event.key];
    if (event.key === "Escape" || event.key === "Tab") {
      event.preventDefault();
      onClose();
    } else if (step) {
      event.preventDefault();
      buttons[(index + step + buttons.length) % buttons.length]?.focus();
    }
  };

  return createPortal(
    <ul
      ref={ref}
      role="menu"
      style={position}
      className="fixed z-50 min-w-44 rounded-md border border-slate-700 bg-slate-900 py-1 text-sm text-slate-100 shadow-xl"
      onKeyDown={onKeyDown}
    >
      {items.map((item) => (
        <li key={item.label} role="none">
          <button
            type="button"
            role="menuitem"
            className="w-full px-3 py-1.5 text-left hover:bg-slate-700 focus:bg-slate-700 focus:outline-none"
            onClick={() => {
              onClose();
              item.onSelect();
            }}
          >
            {item.label}
          </button>
        </li>
      ))}
    </ul>,
    document.body
  );
}
