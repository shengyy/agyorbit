import { type RefObject, useEffect } from "react";
import { ipc } from "../ipc";

/** Keeps the macOS popover exactly as tall as its content. */
export function useAutosize(ref: RefObject<HTMLElement | null>, enabled: boolean) {
  useEffect(() => {
    const element = ref.current;
    if (!enabled || !element) return;
    let frame = 0;
    let last = 0;
    const observer = new ResizeObserver(() => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const height = Math.ceil(element.getBoundingClientRect().height);
        if (height !== last) {
          last = height;
          ipc.resizePanel(height);
        }
      });
    });
    observer.observe(element);
    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
    };
  }, [ref, enabled]);
}
