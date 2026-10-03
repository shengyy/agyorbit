# Architecture

AgyOrbit is a [Tauri 2](https://tauri.app) app: a Rust backend that owns every side effect and a React
frontend that only renders what the backend reports. External facts about Antigravity live in
[antigravity-integration.md](antigravity-integration.md); this page covers how the code is organized.

## Data flow

```text
 tray click ─▶ shell (popover / window) ─▶ React UI
                                            │  invoke(command)        ▲ "agyorbit://snapshot"
                                            ▼                         │
                                  commands.rs ─▶ accounts / switcher / quota / scheduler
                                                         │
                     ┌───────────────────────────────────┼─────────────────────────────┐
                     ▼                                   ▼                             ▼
          antigravity/* (install, live          google/* (OAuth, loopback,     vault + registry
          credential, processes, client)        Cloud Code quota)              (per-account state)
```

The backend keeps one `Orbit` state (`state.rs`). Every mutation goes through `Orbit::update`, which
emits a full `Snapshot` (`model.rs`) to the frontend. The frontend never holds tokens and never decides
anything the backend can know: it renders the latest snapshot and calls commands.

## Backend modules (`src-tauri/src`)

| Module | Owns |
|---|---|
| `lib.rs` | Plugin wiring, startup, command registration |
| `commands.rs` | The frontend's entry points; each delegates to one module |
| `state.rs` | Shared state, snapshot emission, the single operation slot, token minting |
| `model.rs` | Types sent to the frontend |
| `accounts.rs` | Browser sign-in, removal, adopting a sign-in made inside Antigravity |
| `switcher.rs` | Switching accounts, stopping and restarting Antigravity |
| `quota.rs` | Refreshing quota and plan for each account |
| `scheduler.rs` | The 5-minute background pass |
| `registry.rs` | Account metadata in `accounts.json` (no secrets) |
| `vault.rs` | Per-account refresh tokens in the credential store |
| `tokens.rs` | In-memory access tokens |
| `secret_store/` | macOS Keychain / Windows Credential Manager primitives |
| `antigravity/` | Install location, OAuth client discovery, credential format, live credential, processes |
| `google/` | OAuth (PKCE), loopback callback server, Cloud Code endpoints, User-Agent |
| `shell.rs` / `tray.rs` | The popover or window, and the tray icon |
| `error.rs` | Error codes the frontend localizes |
| `system_tests.rs` | Read-only checks against a real installation (`--ignored`) |

## Frontend (`src`)

| Path | Owns |
|---|---|
| `types.ts` | Mirror of `model.rs` |
| `ipc.ts` | Command and event names |
| `app.tsx` | Composition and the dialog state machine |
| `menus.ts` | Native context menus |
| `quota.ts` | Derived facts: severity, usable share, best account to switch to |
| `i18n.ts` / `format.ts` | Strings (English, Simplified Chinese) and time / percent formatting |
| `hooks/` | Snapshot subscription, popover auto-size, relative-time ticker |
| `components/` | One component per file, each with its own stylesheet |
| `styles/` | Design tokens and base styles |

## Operations

One operation runs at a time (`Orbit::begin` returns `Busy` otherwise): `adding`, `switching` (with its
current step), `stopping`, `restarting`. The background pass skips while one is running.

### Switch sequence

1. **Preparing**: refresh the target's token. If Google refuses (`invalid_grant`), stop here; nothing has
   been touched, and the account is marked for re-authorization.
2. Save the current sign-in back to its account's vault entry (Antigravity may hold a newer token).
3. **Closing**: stop every Antigravity process.
4. **Writing**: write keychain + fallback file, read back and verify. On failure restore the previous
   credential (or clear it if there was none) and relaunch.
5. **Launching**: open Antigravity and wait for its main process.

### Adopting a sign-in made in the IDE

`accounts.json` keeps a fingerprint (truncated SHA-256) of the last refresh token seen in Antigravity's
credential. A different fingerprint means a new sign-in: the account is adopted once. A removed account
therefore stays removed until Antigravity signs in again.

## Storage

| What | Where |
|---|---|
| Account metadata | `<app data>/accounts.json` (`~/Library/Application Support/io.github.shengyy.agyorbit/`, `%APPDATA%\io.github.shengyy.agyorbit\`) |
| Refresh tokens | Credential store, service `agyorbit`, account = Google `sub` |
| Access tokens | Memory only |
| Logs | `<app log dir>` (`~/Library/Logs/io.github.shengyy.agyorbit/`), never containing tokens |
