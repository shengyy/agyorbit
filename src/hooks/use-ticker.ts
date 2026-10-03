import { useEffect, useState } from "react";

/** Re-renders every `ms` so relative times ("2 min ago") stay current. */
export function useTicker(ms = 30_000): number {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), ms);
    return () => window.clearInterval(timer);
  }, [ms]);
  return now;
}
