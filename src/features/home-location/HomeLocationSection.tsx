import { Button } from "../../components/design-system/Button";
import type { HomeLocation } from "../../lib/types";
import { HomeCitySearch } from "./HomeCitySearch";
import { homeFromCandidate, homeStatus } from "./homeLabels";

interface HomeLocationSectionProps {
  home: HomeLocation | null;
  disabled: boolean;
  onChange: (home: HomeLocation | null) => void;
}

// Trips are folders for files taken far from home, so home can be picked or cleared here.
export function HomeLocationSection({ home, disabled, onChange }: HomeLocationSectionProps) {
  return (
    <section
      aria-labelledby="home-location-heading"
      className="mt-4 border-t border-slate-800 pt-3"
    >
      <h2 id="home-location-heading" className="text-sm font-semibold text-slate-100">
        Home
      </h2>
      <p className="mt-1 text-xs text-slate-400">
        Files taken far from home are suggested as trips. A large scan sets it for you.
      </p>
      <div className="mt-2 flex items-center justify-between gap-2">
        <p className="text-xs text-slate-200">{homeStatus(home)}</p>
        <Button
          variant="secondary"
          className="px-2 py-1 text-xs disabled:text-slate-500"
          disabled={disabled || !home}
          aria-label="Clear home"
          onClick={() => onChange(null)}
        >
          Clear
        </Button>
      </div>
      <HomeCitySearch disabled={disabled} onPick={(city) => onChange(homeFromCandidate(city))} />
    </section>
  );
}
