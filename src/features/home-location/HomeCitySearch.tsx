import { useId, useState, type KeyboardEvent } from "react";
import type { HomeCandidate } from "../../lib/types";
import { candidateLabel } from "./homeLabels";
import { useHomeCitySearch } from "./useHomeCitySearch";

interface HomeCitySearchProps {
  disabled: boolean;
  onPick: (city: HomeCandidate) => void;
}

// A combobox: type a city, move through matches with the arrow keys, Enter picks one.
export function HomeCitySearch({ disabled, onPick }: HomeCitySearchProps) {
  const [text, setText] = useState("");
  const [active, setActive] = useState(-1);
  const { cities, error, searching } = useHomeCitySearch(text);
  const id = useId();
  const listId = `${id}-cities`;
  const expanded = cities.length > 0;
  const activeIndex = active < cities.length ? active : -1;

  const pick = (city: HomeCandidate) => {
    onPick(city);
    setText("");
    setActive(-1);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "ArrowDown" && expanded) {
      event.preventDefault();
      setActive((activeIndex + 1) % cities.length);
    } else if (event.key === "ArrowUp" && expanded) {
      event.preventDefault();
      setActive(activeIndex <= 0 ? cities.length - 1 : activeIndex - 1);
    } else if (event.key === "Enter" && activeIndex >= 0) {
      event.preventDefault();
      pick(cities[activeIndex]);
    } else if (event.key === "Escape" && text !== "") {
      // Clears the search first; the next Escape closes the settings pane.
      event.stopPropagation();
      setText("");
      setActive(-1);
    }
  };

  return (
    <div className="mt-2">
      <label htmlFor={id} className="block text-xs text-slate-400">
        Find a city
      </label>
      <input
        id={id}
        type="text"
        role="combobox"
        autoComplete="off"
        aria-autocomplete="list"
        aria-controls={listId}
        aria-expanded={expanded}
        aria-activedescendant={activeIndex >= 0 ? `${listId}-${activeIndex}` : undefined}
        className="mt-1 w-full rounded border border-slate-600 bg-slate-950 px-2 py-1 text-xs text-slate-100 focus-visible:outline-2 focus-visible:outline-sky-400"
        disabled={disabled}
        value={text}
        onChange={(event) => {
          setText(event.target.value);
          setActive(-1);
        }}
        onKeyDown={onKeyDown}
      />
      <ul
        id={listId}
        role="listbox"
        aria-label="Cities"
        className="mt-1 max-h-48 overflow-y-auto rounded border border-slate-700 bg-slate-950 empty:hidden"
      >
        {cities.map((city, index) => (
          <li
            key={`${city.name}:${city.latitude}:${city.longitude}`}
            id={`${listId}-${index}`}
            role="option"
            aria-selected={index === activeIndex}
            className="cursor-pointer px-2 py-1 text-xs text-slate-200 hover:bg-slate-800 aria-selected:bg-sky-700 aria-selected:text-white"
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => pick(city)}
          >
            {candidateLabel(city)}
          </li>
        ))}
      </ul>
      <p aria-live="polite" className="mt-1 text-xs text-slate-400 empty:hidden">
        {searchStatus(error, searching, text.trim() !== "" && !expanded)}
      </p>
    </div>
  );
}

function searchStatus(error: string | null, searching: boolean, noMatches: boolean): string {
  if (error) {
    return `Search failed: ${error}`;
  }
  if (searching) {
    return "Searching…";
  }
  return noMatches ? "No cities found." : "";
}
