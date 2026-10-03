import { useEffect, useRef } from "react";
import { t } from "../i18n";
import { Sheet, SheetActions } from "./sheet";

interface Props {
  title: string;
  body: string;
  note?: string | null;
  action: string;
  destructive?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmSheet({ title, body, note, action, destructive, onConfirm, onCancel }: Props) {
  const primary = useRef<HTMLButtonElement>(null);
  useEffect(() => primary.current?.focus(), []);
  return (
    <Sheet onDismiss={onCancel} onConfirm={onConfirm}>
      <h2>{title}</h2>
      <p>{body}</p>
      {note && <p className="sheet-note">{note}</p>}
      <SheetActions>
        <button type="button" className="btn" onClick={onCancel}>
          {t("confirm.cancel")}
        </button>
        <button
          type="button"
          className={destructive ? "btn btn-danger" : "btn btn-primary"}
          onClick={onConfirm}
          ref={primary}
        >
          {action}
        </button>
      </SheetActions>
    </Sheet>
  );
}
