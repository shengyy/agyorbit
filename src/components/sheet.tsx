import { type ReactNode, useEffect } from "react";
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
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape" && onDismiss) {
        event.stopPropagation();
        onDismiss();
      }
      if (event.key === "Enter" && onConfirm && !(event.target instanceof HTMLButtonElement)) {
        event.preventDefault();
        onConfirm();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [onDismiss, onConfirm]);

  return (
    <div className="sheet-layer">
      <button
        type="button"
        className="sheet-backdrop"
        tabIndex={-1}
        aria-label="Dismiss"
        onClick={onDismiss}
      />
      <div className={`sheet ${className}`} role="dialog" aria-modal="true">
        {children}
      </div>
    </div>
  );
}

export function SheetActions({ children }: { children: ReactNode }) {
  return <div className="sheet-actions">{children}</div>;
}
