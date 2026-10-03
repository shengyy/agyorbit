import { useEffect, useState } from "react";
import { ipc, onSnapshot } from "../ipc";
import type { Snapshot } from "../types";

/** The latest backend snapshot; events win over the initial fetch. */
export function useSnapshot(): Snapshot | null {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  useEffect(() => {
    let disposed = false;
    const unlisten = onSnapshot((next) => {
      if (!disposed) setSnapshot(next);
    });
    ipc.snapshot().then((initial) => {
      if (!disposed) setSnapshot((current) => current ?? initial);
    });
    return () => {
      disposed = true;
      unlisten.then((stop) => stop());
    };
  }, []);
  return snapshot;
}
