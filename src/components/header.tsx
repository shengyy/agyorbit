import type { MouseEvent } from "react";
import { ago } from "../format";
import { t } from "../i18n";
import type { Snapshot } from "../types";
import { MoreIcon, OrbitMark, RefreshIcon } from "./icons";
import "./header.css";

interface Props {
  snapshot: Snapshot;
  onRefresh: () => void;
  onMore: (event: MouseEvent) => void;
}

export function Header({ snapshot, onRefresh, onMore }: Props) {
  const { antigravity, refreshing, refreshedAt } = snapshot;
  const status = !antigravity.installed ? "missing" : antigravity.running ? "running" : "stopped";
  const updated = refreshing
    ? t("header.refreshing")
    : refreshedAt
      ? t("header.updated", { ago: ago(refreshedAt) })
      : null;
  return (
    <header className="panel-header">
      <OrbitMark className="brand-mark" width={22} height={22} />
      <div className="brand">
        <div className="brand-name">AgyOrbit</div>
        <div className="brand-status">
          <span className={`status-dot status-${status}`} />
          <span className="truncate">
            {t(`status.${status}`)}
            {updated && ` · ${updated}`}
          </span>
        </div>
      </div>
      <button
        type="button"
        className={`icon-btn${refreshing ? " spinning" : ""}`}
        aria-label={t("header.refresh")}
        title={t("header.refresh")}
        disabled={refreshing}
        onClick={onRefresh}
      >
        <RefreshIcon />
      </button>
      <button
        type="button"
        className="icon-btn"
        aria-label={t("header.more")}
        title={t("header.more")}
        onClick={onMore}
      >
        <MoreIcon />
      </button>
    </header>
  );
}
