import { useCallback, useState } from "react";
import { Button } from "../../components/design-system/Button";
import { Dialog } from "../../components/design-system/Dialog";
import type { MetadataResult, PlaceSuggestion } from "../../lib/types";
import { freshChoice, isChoiceValid, type PlaceChoice } from "./placeChoice";
import { PlaceSuggestionSection } from "./PlaceSuggestionSection";

interface PlaceSuggestionsDialogProps {
  suggestions: PlaceSuggestion[];
  choices: Record<string, PlaceChoice>;
  metadataByFileId: Record<string, MetadataResult>;
  onApply: (choices: Record<string, PlaceChoice>) => void;
  onCancel: () => void;
}

export function PlaceSuggestionsDialog({
  suggestions,
  choices,
  metadataByFileId,
  onApply,
  onCancel
}: PlaceSuggestionsDialogProps) {
  const [draft, setDraft] = useState<Record<string, PlaceChoice>>(() =>
    Object.fromEntries(suggestions.map((s) => [s.id, choices[s.id] ?? freshChoice(s)]))
  );
  const choiceFor = (suggestion: PlaceSuggestion) =>
    draft[suggestion.id] ?? freshChoice(suggestion);
  const valid = suggestions.every((suggestion) => isChoiceValid(suggestion, choiceFor(suggestion)));
  const dateOf = useCallback(
    (fileId: string) => metadataByFileId[fileId]?.chosen_date,
    [metadataByFileId]
  );

  return (
    <Dialog
      title="Edit place folders"
      wide
      onCancel={onCancel}
      actions={
        <>
          <Button variant="secondary" className="px-3 py-1.5 text-sm" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            variant="primary"
            className="px-3 py-1.5 text-sm"
            disabled={!valid}
            onClick={() => onApply(draft)}
          >
            Update folders
          </Button>
        </>
      }
    >
      <p className="mb-4">
        Rename the folders or send a trip to date folders. Files move when you press Move Files.
      </p>
      {suggestions.map((suggestion) => (
        <PlaceSuggestionSection
          key={suggestion.id}
          suggestion={suggestion}
          choice={choiceFor(suggestion)}
          dateOf={dateOf}
          onChange={(choice) => setDraft((previous) => ({ ...previous, [suggestion.id]: choice }))}
        />
      ))}
      <p className="mt-4 text-xs text-slate-400">
        Place names from GeoNames and © OpenStreetMap contributors.
      </p>
    </Dialog>
  );
}
