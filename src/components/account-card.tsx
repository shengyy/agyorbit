import type { MouseEvent } from "react";
import { displayName, untilReset } from "../format";
import { errorText, t } from "../i18n";
import type { Account, QuotaGroup } from "../types";
import { Avatar } from "./avatar";
import { MoreIcon, WarningIcon } from "./icons";
import { PlanBadge } from "./plan-badge";
import { QuotaRow } from "./quota-meter";
import "./account-card.css";

interface Props {
  account: Account;
  active: boolean;
  best: boolean;
  disabled: boolean;
  onSwitch: () => void;
  onReauth: () => void;
  onMenu: (event: MouseEvent) => void;
}

const ORDER = { gemini: 0, claude: 1, other: 2 } as const;

/** The tightest low window that will recover, e.g. "Claude 5h · resets in 2h 14m". */
function recoveryHint(groups: QuotaGroup[]): string | null {
  const candidates = groups.flatMap((group) =>
    [
      { group, label: t("quota.fiveHour"), window: group.fiveHour },
      { group, label: t("quota.weekly"), window: group.weekly },
    ].filter((c) => c.window && c.window.remaining < 0.35),
  );
  candidates.sort((a, b) => (a.window?.remaining ?? 1) - (b.window?.remaining ?? 1));
  for (const { group, label, window } of candidates) {
    const time = untilReset(window?.resetAt ?? null);
    if (time) return `${group.label} ${label} · ${t("quota.resetsIn", { time })}`;
  }
  return null;
}

function QuotaBody({ account, onReauth }: Pick<Props, "account" | "onReauth">) {
  const { quota } = account;
  switch (quota.status) {
    case "loading":
      return (
        <div className="card-quota" aria-busy="true">
          <div className="skeleton" />
          <div className="skeleton" />
        </div>
      );
    case "reauth":
      return (
        <div className="card-notice">
          <WarningIcon />
          <span>{t("account.reauth")}</span>
          <button type="button" className="link-btn" onClick={onReauth}>
            {t("account.reauthAction")}
          </button>
        </div>
      );
    case "failed":
      return (
        <div className="card-notice" title={quota.message}>
          <WarningIcon />
          <span>{errorText(quota.code, quota.message)}</span>
        </div>
      );
    case "ready": {
      const groups = [...quota.groups].sort((a, b) => ORDER[a.key] - ORDER[b.key]);
      const hint = recoveryHint(groups);
      return (
        <div className="card-quota">
          {groups.map((group) => (
            <QuotaRow key={group.key + group.label} group={group} />
          ))}
          {hint && <div className="card-hint">{hint}</div>}
        </div>
      );
    }
  }
}

export function AccountCard({ account, active, best, disabled, onSwitch, onReauth, onMenu }: Props) {
  const name = displayName(account.name, account.email);
  const canSwitch = !active && !disabled && account.quota.status !== "reauth";
  const classes = ["card", active && "card-active", canSwitch && "card-switchable"].filter(Boolean).join(" ");
  return (
    // biome-ignore lint/a11y/useKeyWithClickEvents: the Switch button inside is the keyboard path.
    <article
      className={classes}
      onClick={canSwitch ? onSwitch : undefined}
      onContextMenu={(event) => {
        event.preventDefault();
        onMenu(event);
      }}
    >
      <header className="card-head">
        <Avatar id={account.id} label={name} picture={account.picture} />
        <div className="card-id">
          <div className="card-name">
            <span className="truncate">{name}</span>
            <PlanBadge plan={account.plan} />
          </div>
          <div className="card-email truncate">{account.email}</div>
        </div>
        {active && <span className="tag tag-current">{t("account.current")}</span>}
        {!active && best && <span className="tag tag-best">{t("account.best")}</span>}
        {canSwitch && (
          <button
            type="button"
            className="switch-btn"
            onClick={(event) => {
              event.stopPropagation();
              onSwitch();
            }}
          >
            {t("account.switch")}
          </button>
        )}
        <button
          type="button"
          className="icon-btn card-more"
          aria-label={t("account.actions")}
          onClick={(event) => {
            event.stopPropagation();
            onMenu(event);
          }}
        >
          <MoreIcon />
        </button>
      </header>
      <QuotaBody account={account} onReauth={onReauth} />
    </article>
  );
}
