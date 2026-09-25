import { useEffect, useState } from "react";

function matches(query: string): boolean {
  return typeof window !== "undefined" && window.matchMedia(query).matches;
}

/* Reads the query synchronously on first render so layout that depends on it
   does not paint the wrong arrangement for a frame before correcting itself. */
export const useMediaQuery = (query: string): boolean => {
  const [isMatch, setIsMatch] = useState(() => matches(query));

  useEffect(() => {
    const mediaQuery = window.matchMedia(query);
    setIsMatch(mediaQuery.matches);

    const handleChange = () => setIsMatch(mediaQuery.matches);
    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [query]);

  return isMatch;
};
