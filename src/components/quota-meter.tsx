import { clock, percent, untilReset } from "../format";
import { t } from "../i18n";
import { severity } from "../quota";
import type { QuotaGroup, QuotaWindow } from "../types";
import "./quota-meter.css";

function Meter({ label, window }: { label: string; window: QuotaWindow | null }) {
  if (!window) {
    return (
      <div className="meter meter-empty">
        <span className="meter-label">{label}</span>
        <span className="meter-track" />
        <span className="meter-value">–</span>
        <span className="meter-reset">{t("quota.unavailable")}</span>
      </div>
    );
  }
  const resetAt = window.resetAt && Number.isFinite(Date.parse(window.resetAt)) ? window.resetAt : null;
  const resetsIn = untilReset(resetAt);
  const resetLabel = resetsIn
    ? t("quota.resetsIn", { time: resetsIn })
    : t(resetAt ? "quota.resetPending" : "quota.resetUnknown");
  const tooltip = resetAt ? `${t("quota.resetsAt", { time: clock(resetAt) })} · ${resetLabel}` : resetLabel;
  return (
    <div className={`meter meter-${severity(window.remaining)}`} title={tooltip}>
      <span className="meter-label">{label}</span>
      <span className="meter-track">
        <span className="meter-fill" style={{ width: `${Math.max(window.remaining * 100, 2)}%` }} />
      </span>
      <span className="meter-value">{percent(window.remaining)}</span>
      <span className="meter-reset">{resetLabel}</span>
    </div>
  );
}

/** One model group: its 5-hour and weekly windows side by side. */
export function QuotaRow({ group }: { group: QuotaGroup }) {
  return (
    <div
      className="quota-row"
      title={group.models.length ? t("quota.models", { list: group.models.join(", ") }) : undefined}
    >
      <span className="quota-group">{group.label}</span>
      <Meter label={t("quota.fiveHour")} window={group.fiveHour} />
      <Meter label={t("quota.weekly")} window={group.weekly} />
    </div>
  );
}
