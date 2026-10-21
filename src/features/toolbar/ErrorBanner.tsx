import { Button } from "../../components/design-system/Button";

interface ErrorBannerProps {
  message: string;
  onDismiss: () => void;
}

export function ErrorBanner({ message, onDismiss }: ErrorBannerProps) {
  return (
    <div
      role="alert"
      className="flex items-start gap-3 border-b border-rose-900 bg-rose-950/60 px-3 py-2 text-sm text-rose-200"
    >
      <p className="min-w-0 flex-1 break-words">{message}</p>
      <Button
        variant="secondary"
        className="shrink-0 bg-transparent px-2 py-0.5 text-xs text-rose-200 hover:bg-rose-900"
        onClick={onDismiss}
      >
        Dismiss
      </Button>
    </div>
  );
}
