import { type ReactNode, useEffect, useLayoutEffect, useRef } from "react";
import "./sheet.css";

interface Props {
  children: ReactNode;
  className?: string;
  /** Escape and clicks on the backdrop call this; omit to make the sheet modal. */
  onDismiss?: () => void;
  onConfirm?: () => void;
}

/** A card that slides up over the panel for confirmations and progress. */
export function Sheet({ children, className = "", onDismiss, onConfirm }: Props) {
  const layer = useRef<HTMLDivElement>(null);
  const sheet = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const siblings = [...(layer.current?.parentElement?.children ?? [])]
      .filter((node): node is HTMLElement => node instanceof HTMLElement && node !== layer.current)
      .map((node) => ({ node, inert: node.inert }));
    for (const { node } of siblings) node.inert = true;
    sheet.current?.focus({ preventScroll: true });
    return () => {
      for (const { node, inert } of siblings) node.inert = inert;
      if (previous?.isConnected) previous.focus({ preventScroll: true });
    };
  }, []);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Tab") {
        const focusable = [
          ...(sheet.current?.querySelectorAll<HTMLElement>(
            'button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
          ) ?? []),
        ].filter((node) => node.getClientRects().length > 0);
        const first = focusable[0];
        const last = focusable[focusable.length - 1];
        if (!first) {
          event.preventDefault();
          sheet.current?.focus();
        } else if (!sheet.current?.contains(document.activeElement)) {
          event.preventDefault();
          (event.shiftKey ? last : first)?.focus();
        } else if (
          event.shiftKey &&
          (document.activeElement === first || document.activeElement === sheet.current)
        ) {
          event.preventDefault();
          last?.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first.focus();
        }
      }
      if (event.key === "Escape" && onDismiss) {
        event.stopPropagation();
        onDismiss();
      }
      if (event.key === "Enter" && onConfirm && event.target === sheet.current) {
        event.preventDefault();
        onConfirm();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [onDismiss, onConfirm]);

  return (
    <div className="sheet-layer" ref={layer}>
      <button
        type="button"
        className="sheet-backdrop"
        tabIndex={-1}
        aria-label="Dismiss"
        onClick={onDismiss}
      />
      <div className={`sheet ${className}`} ref={sheet} tabIndex={-1} role="dialog" aria-modal="true">
        {children}
      </div>
    </div>
  );
}

export function SheetActions({ children }: { children: ReactNode }) {
  return <div className="sheet-actions">{children}</div>;
}
