import { useState } from "react";
import { Button } from "../../components/design-system/Button";
import type { HomeLocation } from "../../lib/types";
import { HomeCitySearch } from "./HomeCitySearch";
import { homeFromCandidate, homeSourceLabel, homeStatus } from "./homeLabels";

interface HomeLocationSectionProps {
  home: HomeLocation | null;
  disabled: boolean;
  onChange: (home: HomeLocation | null) => void;
  onLocate: () => Promise<void>;
}

const smallButton = "px-2 py-1 text-xs disabled:text-slate-500";

// Places in home's country get no country folder, so home can be picked, located or cleared here.
export function HomeLocationSection({
  home,
  disabled,
  onChange,
  onLocate
}: HomeLocationSectionProps) {
  const [locating, setLocating] = useState(false);

  const locate = async () => {
    setLocating(true);
    try {
      await onLocate();
    } finally {
      setLocating(false);
    }
  };

  return (
    <section
      aria-labelledby="home-location-heading"
      className="mt-4 border-t border-slate-800 pt-3"
    >
      <h2 id="home-location-heading" className="text-sm font-semibold text-slate-100">
        Home
      </h2>
      <p className="mt-1 text-xs text-slate-400">
        Places outside home's country go in a country folder. Home comes from your location, your
        time zone or a large scan.
      </p>
      <div className="mt-2 flex items-center justify-between gap-2">
        <p className="text-xs text-slate-200" aria-live="polite">
          {homeStatus(home)}
          {home ? <span className="text-slate-400"> · {homeSourceLabel(home.source)}</span> : null}
        </p>
        <Button
          variant="secondary"
          className={smallButton}
          disabled={disabled || !home}
          aria-label="Clear home"
          onClick={() => onChange(null)}
        >
          Clear
        </Button>
      </div>
      <Button
        variant="secondary"
        className={`mt-2 ${smallButton}`}
        disabled={disabled}
        onClick={() => void locate()}
      >
        {locating ? "Locating…" : "Use current location"}
      </Button>
      <HomeCitySearch disabled={disabled} onPick={(city) => onChange(homeFromCandidate(city))} />
    </section>
  );
}
