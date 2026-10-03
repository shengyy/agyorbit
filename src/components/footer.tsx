import { t } from "../i18n";
import type { Snapshot } from "../types";
import { PlayIcon, PlusIcon, PowerIcon, RestartIcon } from "./icons";
import "./footer.css";

interface Props {
  snapshot: Snapshot;
  busy: boolean;
  onAdd: () => void;
  onRestart: () => void;
  onStop: () => void;
}

export function Footer({ snapshot, busy, onAdd, onRestart, onStop }: Props) {
  const { installed, running } = snapshot.antigravity;
  return (
    <footer className="panel-footer">
      <button type="button" className="footer-btn footer-add" disabled={busy} onClick={onAdd}>
        <PlusIcon />
        {t("footer.add")}
      </button>
      <span className="footer-spacer" />
      {installed && running && (
        <>
          <button
            type="button"
            className="footer-btn"
            title={t("footer.restartTitle")}
            disabled={busy}
            onClick={onRestart}
          >
            <RestartIcon />
            {t("footer.restart")}
          </button>
          <button
            type="button"
            className="footer-btn"
            title={t("footer.stopTitle")}
            disabled={busy}
            onClick={onStop}
          >
            <PowerIcon />
            {t("footer.stop")}
          </button>
        </>
      )}
      {installed && !running && (
        <button type="button" className="footer-btn" disabled={busy} onClick={onRestart}>
          <PlayIcon />
          {t("footer.launch")}
        </button>
      )}
    </footer>
  );
}
