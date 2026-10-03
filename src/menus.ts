// Native context menus, so they look and behave like the platform's own.

import { LogicalPosition } from "@tauri-apps/api/dpi";
import { CheckMenuItem, Menu } from "@tauri-apps/api/menu";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { MouseEvent } from "react";
import { t } from "./i18n";
import { ipc } from "./ipc";
import type { Account } from "./types";

const REPOSITORY = "https://github.com/shengyy/agyorbit";

function anchor(event: MouseEvent): LogicalPosition {
  const target = event.currentTarget as HTMLElement | null;
  if (event.type === "click" && target) {
    const rect = target.getBoundingClientRect();
    return new LogicalPosition(rect.left, rect.bottom + 4);
  }
  return new LogicalPosition(event.clientX, event.clientY);
}

export async function showAppMenu(event: MouseEvent, version: string) {
  const at = anchor(event);
  const launchAtLogin = await CheckMenuItem.new({
    text: t("menu.launchAtLogin"),
    checked: await isEnabled(),
    action: async () => ((await isEnabled()) ? disable() : enable()),
  });
  const menu = await Menu.new({
    items: [
      { text: `AgyOrbit ${version}`, enabled: false },
      { item: "Separator" },
      launchAtLogin,
      { text: t("menu.logs"), action: () => void ipc.revealLogs() },
      { text: t("menu.github"), action: () => void openUrl(REPOSITORY) },
      { item: "Separator" },
      { text: t("menu.quit"), action: () => void ipc.quit() },
    ],
  });
  await menu.popup(at);
}

interface AccountActions {
  onReauth: () => void;
  onRemove: () => void;
}

export async function showAccountMenu(event: MouseEvent, account: Account, actions: AccountActions) {
  const at = anchor(event);
  const menu = await Menu.new({
    items: [
      { text: t("menu.reauth"), action: actions.onReauth },
      { text: t("menu.copyEmail"), action: () => void navigator.clipboard.writeText(account.email) },
      { item: "Separator" },
      { text: t("menu.remove"), action: actions.onRemove },
    ],
  });
  await menu.popup(at);
}
