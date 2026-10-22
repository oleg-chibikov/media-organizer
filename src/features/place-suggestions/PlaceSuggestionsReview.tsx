import type { FileIdMap } from "../../lib/fileIdMap";
import type { MetadataResult } from "../../lib/types";
import { PlaceSuggestionsDialog } from "./PlaceSuggestionsDialog";
import type { PlaceSuggestionsState } from "./usePlaceSuggestions";

interface PlaceSuggestionsReviewProps {
  places: PlaceSuggestionsState;
  metadataByFileId: FileIdMap<MetadataResult>;
}

// The review dialog while it is open, wired to the suggestions state.
export function PlaceSuggestionsReview({ places, metadataByFileId }: PlaceSuggestionsReviewProps) {
  if (!places.reviewOpen) {
    return null;
  }
  return (
    <PlaceSuggestionsDialog
      suggestions={places.suggestions}
      choices={places.choices}
      metadataByFileId={metadataByFileId}
      onApply={places.applyChoices}
      onCancel={places.closeReview}
    />
  );
}
