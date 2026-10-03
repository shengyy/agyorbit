import { useEffect } from "react";
import { t } from "../i18n";
import { CheckIcon, WarningIcon } from "./icons";
import { Sheet, SheetActions } from "./sheet";
import "./result-sheet.css";

interface Props {
  tone: "success" | "error";
  message: string;
  onClose: () => void;
}

const SUCCESS_LINGER_MS = 1600;

export function ResultSheet({ tone, message, onClose }: Props) {
  useEffect(() => {
    if (tone !== "success") return;
    const timer = window.setTimeout(onClose, SUCCESS_LINGER_MS);
    return () => window.clearTimeout(timer);
  }, [tone, onClose]);

  return (
    <Sheet onDismiss={onClose} onConfirm={onClose}>
      <div className={`result result-${tone}`}>
        <span className="result-icon">{tone === "success" ? <CheckIcon /> : <WarningIcon />}</span>
        <span>{message}</span>
      </div>
      {tone === "error" && (
        <SheetActions>
          <button type="button" className="btn btn-primary" onClick={onClose}>
            {t("progress.ok")}
          </button>
        </SheetActions>
      )}
    </Sheet>
  );
}
