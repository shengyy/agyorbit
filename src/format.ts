import { t } from "./i18n";

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

export function percent(fraction: number): string {
  return `${Math.round(fraction * 100)}%`;
}

/** "2h 14m" style duration until `iso`, or null when it has passed. */
export function untilReset(iso: string | null, now = Date.now()): string | null {
  if (!iso) return null;
  const ms = new Date(iso).getTime() - now;
  if (ms <= 0) return null;
  const d = Math.floor(ms / DAY);
  const h = Math.floor((ms % DAY) / HOUR);
  const m = Math.max(1, Math.floor((ms % HOUR) / MINUTE));
  if (d > 0) return t("time.daysHours", { d, h });
  if (h > 0) return t("time.hoursMinutes", { h, m });
  return t("time.minutes", { m });
}

/** Local wall-clock time of `iso`, with the date when it is not today. */
export function clock(iso: string): string {
  const date = new Date(iso);
  const sameDay = date.toDateString() === new Date().toDateString();
  return new Intl.DateTimeFormat(undefined, {
    ...(sameDay ? {} : { month: "short", day: "numeric" }),
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

export function ago(iso: string, now = Date.now()): string {
  const ms = now - new Date(iso).getTime();
  if (ms < MINUTE) return t("time.justNow");
  if (ms < HOUR) return t("time.minutesAgo", { n: Math.floor(ms / MINUTE) });
  return t("time.hoursAgo", { n: Math.floor(ms / HOUR) });
}

/** Profile name, else the local part of the email. */
export function displayName(name: string | null, email: string): string {
  return name?.trim() || email.split("@")[0] || email;
}
