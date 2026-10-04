import { useEffect, useRef } from "react";
import { t } from "../i18n";
import type { UpdateInfo } from "../types";
import { Sheet, SheetActions } from "./sheet";
import "./update-sheet.css";

export function UpdateBanner({
  update,
  disabled,
  onOpen,
}: {
  update: UpdateInfo;
  disabled: boolean;
  onOpen: () => void;
}) {
  return (
    <button type="button" className="update-banner" disabled={disabled} onClick={onOpen}>
      <span>{t("update.available", { version: update.version })}</span>
      <span className="update-banner-action">{t("update.view")} ›</span>
    </button>
  );
}

export function UpdateSheet({
  update,
  currentVersion,
  onConfirm,
  onCancel,
}: {
  update: UpdateInfo;
  currentVersion: string;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const primary = useRef<HTMLButtonElement>(null);
  useEffect(() => primary.current?.focus(), []);
  return (
    <Sheet className="update-sheet" onDismiss={onCancel} onConfirm={onConfirm}>
      <h2>{t("update.title", { version: update.version })}</h2>
      <p className="sheet-note">
        {currentVersion} → {update.version}
      </p>
      {update.notes && (
        // biome-ignore lint/a11y/noNoninteractiveTabindex: Keyboard users need to focus and scroll long notes.
        <section className="update-notes" aria-label={t("update.notes")} tabIndex={0}>
          {update.notes}
        </section>
      )}
      <p>{t("update.body")}</p>
      <SheetActions>
        <button type="button" className="btn" onClick={onCancel}>
          {t("update.later")}
        </button>
        <button type="button" className="btn btn-primary" ref={primary} onClick={onConfirm}>
          {t("update.action")}
        </button>
      </SheetActions>
    </Sheet>
  );
}

export function CheckingUpdateSheet({ onCancel }: { onCancel: () => void }) {
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => cancel.current?.focus(), []);
  return (
    <Sheet onDismiss={onCancel}>
      <div className="activity-line">
        <span className="spinner" />
        {t("update.checking")}
      </div>
      <SheetActions>
        <button type="button" className="btn" ref={cancel} onClick={onCancel}>
          {t("update.later")}
        </button>
      </SheetActions>
    </Sheet>
  );
}
