import type { MouseEvent } from "react";
import { displayName } from "../format";
import { errorText, t } from "../i18n";
import type { Account } from "../types";
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
      return (
        <div className="card-quota">
          {groups.map((group) => (
            <QuotaRow key={group.key + group.label} group={group} />
          ))}
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
            {active && <span className="tag tag-current">{t("account.current")}</span>}
            {!active && best && <span className="tag tag-best">{t("account.best")}</span>}
          </div>
          <div className="card-email-row">
            <span className="card-email truncate">{account.email}</span>
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
          </div>
        </div>
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
