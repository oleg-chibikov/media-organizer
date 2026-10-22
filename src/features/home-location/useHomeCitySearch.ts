import { useEffect, useState } from "react";
import { searchHomeCities } from "../../lib/api";
import type { HomeCandidate } from "../../lib/types";

// Waits for a pause in typing so each keystroke doesn't search.
const SEARCH_DELAY_MS = 200;

interface SearchAnswer {
  query: string;
  cities: HomeCandidate[];
  error: string | null;
}

const NO_ANSWER: SearchAnswer = { query: "", cities: [], error: null };

// Asks the backend for cities matching the typed text, showing only answers for the current text.
export function useHomeCitySearch(text: string) {
  const query = text.trim();
  const [answer, setAnswer] = useState<SearchAnswer>(NO_ANSWER);

  useEffect(() => {
    if (!query) {
      return;
    }
    let ignore = false;
    const keep = (found: SearchAnswer) => {
      if (!ignore) {
        setAnswer(found);
      }
    };
    const timer = setTimeout(() => {
      searchHomeCities(query).then(
        (cities) => keep({ query, cities, error: null }),
        (error: unknown) => keep({ query, cities: [], error: String(error) })
      );
    }, SEARCH_DELAY_MS);
    return () => {
      ignore = true;
      clearTimeout(timer);
    };
  }, [query]);

  const current = query !== "" && answer.query === query ? answer : null;
  return {
    cities: current?.cities ?? [],
    error: current?.error ?? null,
    searching: query !== "" && current === null
  };
}
