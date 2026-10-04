// Mirrors src-tauri/src/model.rs. The backend is the source of truth.

export type GroupKey = "gemini" | "claude" | "other";
export type PlanKind = "free" | "pro" | "ultra";

export interface QuotaWindow {
  /** 0–1 share still available. */
  remaining: number;
  resetAt: string | null;
}

export interface QuotaGroup {
  key: GroupKey;
  label: string;
  models: string[];
  fiveHour: QuotaWindow | null;
  weekly: QuotaWindow | null;
}

export interface Plan {
  kind: PlanKind;
  name: string;
}

export type QuotaState =
  | { status: "loading" }
  | { status: "ready"; groups: QuotaGroup[]; fetchedAt: string }
  | { status: "failed"; code: string; message: string }
  | { status: "reauth" };

export interface Account {
  id: string;
  email: string;
  name: string | null;
  picture: string | null;
  plan: Plan | null;
  quota: QuotaState;
}

export type SwitchStep = "preparing" | "closing" | "writing" | "launching";

export type Operation =
  | { kind: "idle" }
  | { kind: "adding" }
  | { kind: "switching"; targetId: string; step: SwitchStep }
  | { kind: "stopping" }
  | { kind: "restarting" }
  | {
      kind: "updating";
      step: "downloading" | "verifying" | "installing" | "restarting";
      downloaded: number;
      total: number | null;
    };

export interface UpdateInfo {
  version: string;
  notes: string | null;
}

export interface Snapshot {
  platform: "macos" | "windows" | "linux";
  version: string;
  accounts: Account[];
  activeId: string | null;
  operation: Operation;
  antigravity: { installed: boolean; running: boolean };
  refreshing: boolean;
  refreshedAt: string | null;
  update: { checking: boolean; available: UpdateInfo | null };
}

export type ProcessKind = "app" | "helper" | "languageServer" | "cli";

export interface RelatedProcess {
  pid: number;
  kind: ProcessKind;
}

/** Error shape serialized by src-tauri/src/error.rs. */
export interface BackendError {
  code: string;
  message: string;
}
