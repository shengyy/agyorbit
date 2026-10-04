import { getCurrentWindow } from "@tauri-apps/api/window";
import { type MouseEvent, useCallback, useEffect, useRef, useState } from "react";
import { AccountCard } from "./components/account-card";
import { ActivitySheet } from "./components/activity-sheet";
import { ConfirmSheet } from "./components/confirm-sheet";
import { EmptyState } from "./components/empty-state";
import { Footer } from "./components/footer";
import { Header } from "./components/header";
import { ResultSheet } from "./components/result-sheet";
import { CheckingUpdateSheet, UpdateBanner, UpdateSheet } from "./components/update-sheet";
import { displayName } from "./format";
import { useAutosize } from "./hooks/use-autosize";
import { useSnapshot } from "./hooks/use-snapshot";
import { useTicker } from "./hooks/use-ticker";
import { errorText, t } from "./i18n";
import { asBackendError, ipc, onShown } from "./ipc";
import { showAccountMenu, showAppMenu } from "./menus";
import { bestAccountId } from "./quota";
import type { Account, ProcessKind, RelatedProcess, UpdateInfo } from "./types";
import "./app.css";

type Dialog =
  | { kind: "update"; update: UpdateInfo }
  | { kind: "checkingUpdate" }
  | { kind: "switch"; account: Account; processes: RelatedProcess[] }
  | { kind: "remove"; account: Account }
  | { kind: "stop" | "restart"; processes: RelatedProcess[] }
  | { kind: "result"; tone: "success" | "error"; message: string };

const PROCESS_ORDER: ProcessKind[] = ["app", "languageServer", "helper", "cli"];

function describeProcesses(processes: RelatedProcess[]): string | null {
  const kinds = PROCESS_ORDER.filter((kind) => processes.some((p) => p.kind === kind));
  if (!kinds.length) return null;
  const list = kinds.map((kind) => {
    const count = processes.filter((p) => p.kind === kind).length;
    return kind === "cli" && count > 1 ? `${t("process.cli")} ×${count}` : t(`process.${kind}`);
  });
  return t("confirm.processes", { list: list.join(" · ") });
}

export function App() {
  const snapshot = useSnapshot();
  const panel = useRef<HTMLDivElement>(null);
  const [dialog, setDialog] = useState<Dialog | null>(null);
  useTicker();
  useAutosize(panel, snapshot?.platform === "macos");

  const closeDialog = useCallback(() => setDialog(null), []);
  const fail = useCallback((error: unknown) => {
    const { code, message } = asBackendError(error);
    if (code !== "oauth_cancelled")
      setDialog({ kind: "result", tone: "error", message: errorText(code, message) });
  }, []);

  useEffect(() => {
    const unlisten = onShown(() => {
      ipc.refresh(false).catch(() => {});
      setDialog((current) => (current?.kind === "result" && current.tone === "success" ? null : current));
    });
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  useEffect(() => {
    if (snapshot?.platform !== "macos" || dialog) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") getCurrentWindow().hide();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [snapshot?.platform, dialog]);

  if (!snapshot) return null;

  const busy = snapshot.operation.kind !== "idle";
  const best = bestAccountId(snapshot.accounts, snapshot.activeId);
  // The account in use leads the list; the rest keep the order they were added in.
  const accounts = [...snapshot.accounts].sort(
    (a, b) => Number(b.id === snapshot.activeId) - Number(a.id === snapshot.activeId),
  );

  const addAccount = () => ipc.addAccount().catch(fail);
  const checkUpdate = async () => {
    setDialog({ kind: "checkingUpdate" });
    try {
      const update = await ipc.checkUpdate();
      setDialog((current) =>
        current?.kind !== "checkingUpdate"
          ? current
          : update
            ? { kind: "update", update }
            : { kind: "result", tone: "success", message: t("update.current") },
      );
    } catch (error) {
      const { code, message } = asBackendError(error);
      setDialog((current) =>
        current?.kind === "checkingUpdate"
          ? { kind: "result", tone: "error", message: errorText(code, message) }
          : current,
      );
    }
  };
  const askSwitch = async (account: Account) =>
    setDialog({ kind: "switch", account, processes: await ipc.runningProcesses() });
  const askAntigravity = async (kind: "stop" | "restart") => {
    const processes = await ipc.runningProcesses();
    if (kind === "restart" && !processes.length) {
      ipc.restartAntigravity().catch(fail);
      return;
    }
    setDialog({ kind, processes });
  };

  const confirmDialog = () => {
    if (!dialog || dialog.kind === "result" || dialog.kind === "checkingUpdate") return;
    setDialog(null);
    switch (dialog.kind) {
      case "update":
        ipc.installUpdate(dialog.update.version).catch(fail);
        break;
      case "switch": {
        const label = displayName(dialog.account.name, dialog.account.email);
        ipc
          .switchAccount(dialog.account.id)
          .then(() =>
            setDialog({ kind: "result", tone: "success", message: t("progress.done", { email: label }) }),
          )
          .catch(fail);
        break;
      }
      case "remove":
        ipc.removeAccount(dialog.account.id).catch(fail);
        break;
      case "stop":
        ipc.stopAntigravity().catch(fail);
        break;
      case "restart":
        ipc.restartAntigravity().catch(fail);
        break;
    }
  };

  const accountMenu = (event: MouseEvent, account: Account) =>
    showAccountMenu(event, account, {
      onReauth: addAccount,
      onRemove: () => setDialog({ kind: "remove", account }),
    });

  return (
    <div ref={panel} className={`panel platform-${snapshot.platform}`}>
      <Header
        snapshot={snapshot}
        onRefresh={() => ipc.refresh(true).catch(fail)}
        onMore={(event) =>
          showAppMenu(event, snapshot.version, {
            checking: snapshot.update.checking,
            enabled: !busy,
            onCheck: checkUpdate,
          })
        }
      />

      {snapshot.update.available && (
        <UpdateBanner
          update={snapshot.update.available}
          disabled={busy || snapshot.update.checking}
          onOpen={() =>
            snapshot.update.available && setDialog({ kind: "update", update: snapshot.update.available })
          }
        />
      )}

      {snapshot.accounts.length === 0 ? (
        <EmptyState disabled={busy} onAdd={addAccount} />
      ) : (
        <main className="account-list">
          {accounts.map((account) => (
            <AccountCard
              key={account.id}
              account={account}
              active={account.id === snapshot.activeId}
              best={account.id === best}
              disabled={busy || !snapshot.antigravity.installed}
              onSwitch={() => askSwitch(account)}
              onReauth={addAccount}
              onMenu={(event) => accountMenu(event, account)}
            />
          ))}
        </main>
      )}

      <Footer
        snapshot={snapshot}
        busy={busy}
        onAdd={addAccount}
        onRestart={() => askAntigravity("restart")}
        onStop={() => askAntigravity("stop")}
      />

      {snapshot.operation.kind !== "idle" ? (
        <ActivitySheet
          operation={snapshot.operation}
          accounts={snapshot.accounts}
          onCancelAdd={() => ipc.cancelAddAccount()}
        />
      ) : dialog?.kind === "result" ? (
        <ResultSheet tone={dialog.tone} message={dialog.message} onClose={closeDialog} />
      ) : dialog?.kind === "update" ? (
        <UpdateSheet
          update={dialog.update}
          currentVersion={snapshot.version}
          onConfirm={confirmDialog}
          onCancel={closeDialog}
        />
      ) : dialog?.kind === "checkingUpdate" ? (
        <CheckingUpdateSheet onCancel={closeDialog} />
      ) : dialog?.kind === "switch" ? (
        <ConfirmSheet
          title={t("confirm.switch.title", { email: dialog.account.email })}
          body={t(snapshot.antigravity.running ? "confirm.switch.body" : "confirm.switch.notRunning")}
          note={describeProcesses(dialog.processes)}
          action={t("confirm.switch.action")}
          onConfirm={confirmDialog}
          onCancel={closeDialog}
        />
      ) : dialog?.kind === "remove" ? (
        <ConfirmSheet
          title={t("confirm.remove.title", { email: dialog.account.email })}
          body={t("confirm.remove.body")}
          note={dialog.account.id === snapshot.activeId ? t("confirm.remove.active") : null}
          action={t("confirm.remove.action")}
          destructive
          onConfirm={confirmDialog}
          onCancel={closeDialog}
        />
      ) : dialog?.kind === "stop" || dialog?.kind === "restart" ? (
        <ConfirmSheet
          title={t(`confirm.${dialog.kind}.title`)}
          body={t(`confirm.${dialog.kind}.body`)}
          note={describeProcesses(dialog.processes)}
          action={t(`confirm.${dialog.kind}.action`)}
          destructive={dialog.kind === "stop"}
          onConfirm={confirmDialog}
          onCancel={closeDialog}
        />
      ) : null}
    </div>
  );
}
