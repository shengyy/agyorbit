# Security

AgyOrbit handles Google OAuth tokens for your own accounts. This page owns the data inventory, storage
locations, network sends and vulnerability reporting; implementation mechanisms live in
[docs/architecture.md](docs/architecture.md).

## What is stored

| Data | Location |
|---|---|
| Refresh token per account | macOS Keychain / Windows Credential Manager, service `agyorbit`, keyed by the Google account id |
| Account id, email, name, photo URL, plan, added time and last-seen sign-in fingerprint | `accounts.json` in the app data directory (no secrets) |
| AgyOrbit's per-account access-token cache | Memory only |
| Antigravity's own credential | Left in its [keyring and fallback file](docs/antigravity-integration.md#signed-in-credential); rewritten only when you confirm a switch |
| Logs | The app log directory; never containing tokens |

The app data directory is `~/Library/Application Support/io.github.shengyy.agyorbit/` on macOS and
`%APPDATA%\io.github.shengyy.agyorbit\` on Windows. The app log directory on macOS is
`~/Library/Logs/io.github.shengyy.agyorbit/`. Keychain access is described in
[Credential access](docs/architecture.md#credential-access).

## What is sent, and to whom

Google endpoints: `accounts.google.com` and `oauth2.googleapis.com` (sign-in and token refresh),
`www.googleapis.com/oauth2/v2/userinfo` (name and photo), and `daily-cloudcode-pa.googleapis.com`
(quota and plan). Google requests identify themselves as `AgyOrbit/<version> (antigravity)`.

App update checks fetch `latest.json` from this repository's latest GitHub Release over HTTPS.
Confirmed downloads use GitHub's release asset hosting. Update requests carry no Google tokens or
account information. Tauri verifies the package signature and its signed version against the public
key shipped in the app before installation. There is no AgyOrbit server, analytics or telemetry.

## What AgyOrbit never does with secrets

- Put tokens, client secrets or account emails in logs, files in this repository, or error messages.
- Redistribute Antigravity's OAuth client secret: it is read at runtime from your local Antigravity
  installation.

Product-level limits, such as never proxying model traffic, are the [non-goals](PRODUCT.md#non-goals).

## Reporting a vulnerability

Please use GitHub's private vulnerability reporting
(<https://github.com/shengyy/agyorbit/security/advisories/new>) rather than a public issue. Include the
version, platform, and steps to reproduce, without real tokens.
