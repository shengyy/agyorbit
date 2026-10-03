# Architecture

This page owns AgyOrbit's code map, data flow, switch sequence and persistence mechanisms. External
contracts and verified versions live in [antigravity-integration.md](antigravity-integration.md); data
inventory and storage locations live in [SECURITY.md](../SECURITY.md).

AgyOrbit is a [Tauri 2](https://tauri.app) app: Rust executes side effects and React renders the backend
snapshot, derives presentation data and requests operations through commands and Tauri plugins.

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

The backend keeps one `Orbit` state (`state.rs`). `Orbit::update` publishes state changes as a full
`Snapshot` (`model.rs`). The frontend never holds tokens. It keeps dialog state and derives account
ordering, quota severity and the recommendation from the snapshot. Account, credential and process
operations run in Rust; native menus, clipboard writes and URL opening use Tauri's APIs and plugins.

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
| `autostart.rs` | Open at Login: SMAppService on macOS, the per-user Run key on Windows |
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

## Antigravity adapters

### Credential access

`antigravity/live.rs` reads the keyring first and writes both the keyring and fallback file, matching
the [external storage contract](antigravity-integration.md#signed-in-credential).

`secret_store/macos.rs` invokes `/usr/bin/security`, as Antigravity's go-keyring does, so each item's ACL
stays bound to that system tool rather than AgyOrbit's per-build signature. This avoids keychain prompts
for AgyOrbit and Antigravity.

`antigravity/credential.rs` preserves unknown fields when decoding and re-encoding an existing bundle.
A switch creates a fresh bundle from Google's token grant, keeping the previous `auth_method` or using
`consumer` when there was none.

### OAuth client discovery

`antigravity/oauth_client.rs` scans the installed Antigravity's `bin/` directory for secret candidates,
cuts each to 35 characters because Go packs literals back to back, and probes Google's token endpoint
with a bogus authorization code. It selects the candidate returning `invalid_grant`; the client and
response contract are in [OAuth client](antigravity-integration.md#oauth-client).

### Process control

`antigravity/process.rs` sends SIGTERM (Windows: `taskkill` without `/F`) to the app and CLI, waits up to
8 s, then kills whatever is left. It relaunches with `open <bundle>` (Windows: the executable, detached)
and waits for the main process. The processes that must exit and why are in
[Processes](antigravity-integration.md#processes).

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

## Persistence

`registry.rs` writes metadata to a temporary JSON file and renames it over `accounts.json`.
`vault.rs` delegates refresh-token storage to `secret_store/`; `tokens.rs` caches access tokens in memory.
The data inventory, storage locations and logging boundary are owned by
[SECURITY.md](../SECURITY.md#what-is-stored).
