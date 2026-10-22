import { Spinner } from "../../components/design-system/Spinner";
import { placeBannerLabel } from "./suggestionLabels";
import type { PlaceSuggestionsState } from "./usePlaceSuggestions";

type BannerSource = Pick<
  PlaceSuggestionsState,
  "locating" | "bannerVisible" | "suggestions" | "openReview" | "dismiss"
>;

const bar = "flex items-center gap-3 border-b border-slate-800 bg-slate-900/60 px-3 py-1.5 text-sm";
const focusRing = "rounded focus-visible:outline-2 focus-visible:outline-sky-400";

// One line above the table. The toolbar button announces progress, so this loader stays quiet.
export function PlaceSuggestionsBanner({ places }: { places: BannerSource }) {
  if (places.locating) {
    return (
      <div className={`${bar} text-slate-300`}>
        <Spinner />
        Finding locations…
      </div>
    );
  }
  if (!places.bannerVisible) {
    return null;
  }
  return (
    <section aria-label="Place folder suggestions" className={`${bar} text-slate-200`}>
      <p className="min-w-0 flex-1 truncate">{placeBannerLabel(places.suggestions)}</p>
      <button
        type="button"
        className={`${focusRing} shrink-0 cursor-pointer px-1 text-sky-300 hover:underline`}
        onClick={places.openReview}
      >
        Review folders
      </button>
      <button
        type="button"
        aria-label="Dismiss place folder suggestions"
        className={`${focusRing} shrink-0 cursor-pointer px-1 text-slate-400 hover:text-slate-100`}
        onClick={places.dismiss}
      >
        <span aria-hidden="true">×</span>
      </button>
    </section>
  );
}
