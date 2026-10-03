# Antigravity integration

Everything AgyOrbit relies on that Google does not document. These are observed facts, each with the
version it was verified on. When Antigravity changes, this page and the module named in each section are
the only places to update.

Re-verify on a machine with Antigravity installed and signed in (read-only, prints no secrets):

```bash
cd src-tauri && cargo test system_tests -- --ignored --nocapture
```

| Fact | Verified on | Owner module |
|---|---|---|
| Signed-in credential storage | 2.19.1, macOS 27 | `antigravity/live.rs` |
| Credential bundle format | 2.19.1, macOS 27 | `antigravity/credential.rs` |
| OAuth client and scopes | 2.19.1, macOS 27 | `antigravity/oauth_client.rs`, `google/oauth.rs` |
| Quota and plan endpoints | 2.19.1, macOS 27 | `google/cloudcode.rs` |
| Processes that hold the credential | 2.19.1, macOS 27 | `antigravity/process.rs` |
| Windows equivalents | **not verified** | same modules |

## Signed-in credential

Antigravity's Go language server stores the credential with
[go-keyring](https://github.com/zalando/go-keyring) under service `gemini`, account `antigravity`:

- **macOS**: a generic password in the login keychain, written through `/usr/bin/security`. The value is
  `go-keyring-base64:` followed by base64 of the JSON bundle.
- **Windows** (from go-keyring's documented layout, not yet verified): a generic credential with target
  `gemini:antigravity` whose blob is the raw JSON.

It also mirrors the JSON to `~/.gemini/jetski-standalone-oauth-token` (mode `0600`) and reads that file
when the keyring times out (the binary logs `Keyring LoadStoredToken timed out … falling back to file
storage`). AgyOrbit therefore reads the keyring first and always writes both copies.

AgyOrbit uses `/usr/bin/security` on macOS too, so every item's ACL stays bound to that tool rather than
to AgyOrbit's own per-build signature: no keychain prompts, for AgyOrbit or for Antigravity.

## Credential bundle

```json
{
  "token": {
    "access_token": "…",
    "token_type": "Bearer",
    "refresh_token": "…",
    "expiry": "2026-10-03T17:52:30.076579+08:00"
  },
  "auth_method": "consumer",
  "id_token": "…"
}
```

- `token` is golang.org/x/oauth2's `Token`; `expiry` uses Go's RFC 3339 format with the local offset.
- `auth_method` is `consumer` for personal Google accounts. AgyOrbit keeps whatever value the previous
  sign-in had and falls back to `consumer`. Workspace / Cloud sign-ins are untested.
- `id_token` identifies the account (`sub`, `email`). A refresh-token grant returns a new one.
- Unknown fields are preserved on rewrite.

## OAuth client

The consumer sign-in client is `1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com`
(the `aud` of Antigravity's ID tokens). The binary also embeds a second client used for Google Cloud
sign-in (`884354919052-…`), which AgyOrbit does not use.

The client ID is public. The installed-app secret is **not** stored in this repository: AgyOrbit scans
the binaries in Antigravity's `bin/` directory for `GOCSPX-` strings (Go packs literals back to back, so
each candidate is cut at the fixed 35-character length) and keeps the one for which a token request with
a bogus authorization code fails with `invalid_grant`; a wrong secret fails with `invalid_client`.

Facts the browser sign-in depends on:

- The client is a desktop client: Google accepts `http://127.0.0.1:<any port>/…` redirects.
- Requested scopes are exactly what the IDE's own sign-in is granted (read back via Google's tokeninfo):
  `openid email profile` plus `cloud-platform`, `aicode`, `cclog` and `experimentsandconfigs` under
  `https://www.googleapis.com/auth/`. A token from AgyOrbit is therefore interchangeable with the IDE's.
- `prompt=select_account consent` always shows the account chooser and always returns a refresh token.

## Quota and plan

Base URL: `https://daily-cloudcode-pa.googleapis.com/v1internal` (what Antigravity passes to its language
server as `--cloud_code_endpoint`). Both calls are `POST` with a bearer token.

- **The `User-Agent` must contain `antigravity`.** Without it the API answers 403 "You do not have a valid
  license of this product". AgyOrbit sends `AgyOrbit/<version> (antigravity)`.
- `:retrieveUserQuotaSummary` with `{}` returns `groups[]`, each with `displayName`, a `description` that
  lists the models (`Models within this group: Claude Opus, Claude Sonnet, GPT-OSS`) and `buckets[]` with
  `window` (`5h` or `weekly`), `remainingFraction` and `resetTime`. Observed groups: `Gemini Models` and
  `Claude and GPT models`. The response is proto3 JSON, so a fraction of `0` is **omitted**: a bucket
  without `remainingFraction` is exhausted.
- `:loadCodeAssist` with `{"metadata":{"ideType":"ANTIGRAVITY"}}` returns `paidTier` (`g1-pro-tier` /
  "Google AI Pro", Ultra contains `ultra`) and `currentTier` (`free-tier` without a subscription).

## Processes

Every process that may hold the credential has to exit before a switch, or it could write the previous
account back when it refreshes its token:

- the app: `Antigravity.app/Contents/MacOS/Antigravity` (Windows: `Antigravity.exe`);
- its Electron helpers and the language server(s) under the same install root
  (`Contents/Resources/bin/language_server --standalone …`);
- the `agy` CLI.

AgyOrbit sends SIGTERM (Windows: `taskkill` without `/F`) to the app and CLI, waits up to 8 s, then kills
whatever is left. It relaunches with `open <bundle>` (Windows: the executable, detached). Each new language
server creates an empty file in `~/.gemini/antigravity/crashes/`; that is a marker, not a crash.

## Not touched

`~/Library/Application Support/Antigravity/app_storage.json` keeps
`jetski.onboarding.lastLoginUsername`, which only pre-fills Antigravity's own sign-in screen. AgyOrbit
leaves it alone.
