// The only module that knows command and event names.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { BackendError, RelatedProcess, Snapshot } from "./types";

export const ipc = {
  snapshot: () => invoke<Snapshot>("get_snapshot"),
  refresh: (force: boolean) => invoke<void>("refresh", { force }),
  addAccount: () => invoke<string>("add_account"),
  cancelAddAccount: () => invoke<void>("cancel_add_account"),
  removeAccount: (id: string) => invoke<void>("remove_account", { id }),
  switchAccount: (id: string) => invoke<void>("switch_account", { id }),
  runningProcesses: () => invoke<RelatedProcess[]>("running_processes"),
  stopAntigravity: () => invoke<void>("stop_antigravity"),
  restartAntigravity: () => invoke<void>("restart_antigravity"),
  resizePanel: (height: number) => invoke<void>("resize_panel", { height }),
  revealLogs: () => invoke<void>("reveal_logs"),
  autostartEnabled: () => invoke<boolean>("autostart_enabled"),
  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),
  quit: () => invoke<void>("quit"),
};

export const onSnapshot = (handler: (snapshot: Snapshot) => void): Promise<UnlistenFn> =>
  listen<Snapshot>("agyorbit://snapshot", (event) => handler(event.payload));

export const onShown = (handler: () => void): Promise<UnlistenFn> => listen("agyorbit://shown", handler);

export function asBackendError(error: unknown): BackendError {
  if (error && typeof error === "object" && "code" in error && "message" in error) {
    return error as BackendError;
  }
  return { code: "unexpected", message: String(error) };
}
