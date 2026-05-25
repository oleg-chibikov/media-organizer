import type { ButtonHTMLAttributes, ReactNode } from "react";

type ButtonVariant = "primary" | "secondary";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  children: ReactNode;
}

const baseClasses =
  "cursor-pointer rounded font-medium disabled:cursor-not-allowed disabled:bg-slate-700";

const variantClasses: Record<ButtonVariant, string> = {
  primary: "bg-sky-600 text-white hover:bg-sky-500",
  secondary: "bg-slate-800 text-slate-100 hover:bg-slate-700"
};

export function Button({
  variant = "secondary",
  className,
  type = "button",
  children,
  ...rest
}: ButtonProps) {
  return (
    <button
      type={type}
      className={`${baseClasses} ${variantClasses[variant]} ${className ?? ""}`.trim()}
      {...rest}
    >
      {children}
    </button>
  );
}
