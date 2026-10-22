import type { PlaceSuggestion } from "../../lib/types";
import { placePromptLabel } from "./placePromptLabel";

interface PlaceSuggestionPromptProps {
  suggestions: PlaceSuggestion[];
  onReview: () => void;
  onDismiss: () => void;
}

const focusRing = "rounded focus-visible:outline-2 focus-visible:outline-sky-400";

// A quiet status bar hint; the files keep their date folders until the person applies a suggestion.
export function PlaceSuggestionPrompt({
  suggestions,
  onReview,
  onDismiss
}: PlaceSuggestionPromptProps) {
  if (suggestions.length === 0) {
    return null;
  }
  return (
    <span className="flex items-center gap-1">
      <button
        type="button"
        className={`${focusRing} cursor-pointer px-1 text-sky-300 hover:underline`}
        onClick={onReview}
      >
        {placePromptLabel(suggestions)}
      </button>
      <button
        type="button"
        aria-label="Dismiss place folder suggestions"
        className={`${focusRing} cursor-pointer px-1 text-slate-400 hover:text-slate-100`}
        onClick={onDismiss}
      >
        <span aria-hidden="true">×</span>
      </button>
    </span>
  );
}
