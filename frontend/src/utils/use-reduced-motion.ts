import { useEffect, useState } from "react";

/**
 * Hook to detect if user prefers reduced motion
 * Respects the prefers-reduced-motion media query
 */
export const useReducedMotion = () => {
  const [reduced, setReduced] = useState(false);

  useEffect(() => {
    const mql = window.matchMedia("(prefers-reduced-motion: reduce)");
    setReduced(mql.matches);

    const listener = (e: MediaQueryListEvent) => setReduced(e.matches);
    mql.addEventListener("change", listener);

    return () => mql.removeEventListener("change", listener);
  }, []);

  return reduced;
};