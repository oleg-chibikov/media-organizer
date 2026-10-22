import { useId, useMemo } from "react";
import type { PlaceSuggestion } from "../../lib/types";
import { FolderNameField } from "./FolderNameField";
import { FolderPreviewList } from "./FolderPreviewList";
import { continuedLabel } from "./continuedLabel";
import { hasEditableTitle, withChoiceNames, type PlaceChoice } from "./placeChoice";
import { placeFolderPreview } from "./placeFolderPreview";
import { suggestionHeading, suggestionSize } from "./suggestionLabels";

interface PlaceSuggestionSectionProps {
  suggestion: PlaceSuggestion;
  choice: PlaceChoice;
  dateOf: (fileId: string) => string | null | undefined;
  onChange: (choice: PlaceChoice) => void;
}

export function PlaceSuggestionSection({
  suggestion,
  choice,
  dateOf,
  onChange
}: PlaceSuggestionSectionProps) {
  const headingId = useId();
  const edited = useMemo(() => withChoiceNames(suggestion, choice), [suggestion, choice]);
  const preview = useMemo(() => placeFolderPreview(edited, dateOf), [edited, dateOf]);
  const renamePlace = (index: number, name: string) =>
    onChange({
      ...choice,
      placeNames: choice.placeNames.map((old, i) => (i === index ? name : old))
    });

  return (
    <section
      aria-labelledby={headingId}
      className="mt-4 border-t border-slate-800 pt-4 first:mt-0 first:border-t-0 first:pt-0"
    >
      <h3 id={headingId} className="font-semibold text-slate-100">
        {suggestionHeading(suggestion)}
      </h3>
      <p className="text-xs text-slate-400">{suggestionSize(suggestion)}</p>
      {choice.use && edited.continues ? (
        <p className="text-xs text-sky-300">{continuedLabel(edited.continues)}</p>
      ) : null}
      <label className="mt-2 flex cursor-pointer items-center gap-2 text-slate-200">
        <input
          type="checkbox"
          checked={choice.use}
          aria-describedby={headingId}
          onChange={(event) => onChange({ ...choice, use: event.target.checked })}
        />
        Use place folders
      </label>
      {choice.use ? (
        <>
          <div className="mt-2 grid gap-2">
            {hasEditableTitle(suggestion) ? (
              <FolderNameField
                label="Trip folder"
                value={choice.title}
                onChange={(title) => onChange({ ...choice, title })}
              />
            ) : null}
            {suggestion.places.map((place, index) => (
              <FolderNameField
                key={place.name}
                label={`Folder for ${place.name}`}
                value={choice.placeNames[index] ?? place.name}
                onChange={(name) => renamePlace(index, name)}
              />
            ))}
          </div>
          <FolderPreviewList folders={preview} />
        </>
      ) : (
        <p className="mt-2 text-slate-400">These files go to date folders.</p>
      )}
    </section>
  );
}
