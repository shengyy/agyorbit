# Security

AgyOrbit handles Google OAuth tokens for your own accounts. This page states what it stores, where, and
what it sends.

## What is stored

| Data | Location |
|---|---|
| Refresh token per account | macOS Keychain / Windows Credential Manager, service `agyorbit`, keyed by the Google account id |
| Account email, name, photo URL, plan | `accounts.json` in the app data directory (no secrets) |
| Access tokens | Memory only |
| Antigravity's own credential | Left where Antigravity keeps it; rewritten only when you confirm a switch |

On macOS every keychain item is written through `/usr/bin/security`, exactly as Antigravity's own
go-keyring does.

## What is sent, and to whom

Only Google endpoints: `accounts.google.com` and `oauth2.googleapis.com` (sign-in and token refresh),
`www.googleapis.com/oauth2/v2/userinfo` (name and photo), and `daily-cloudcode-pa.googleapis.com`
(quota and plan). There is no AgyOrbit server, analytics or telemetry. Requests identify themselves as
`AgyOrbit/<version> (antigravity)`.

## What AgyOrbit never does with secrets

- Put tokens, client secrets or account emails in logs, files in this repository, or error messages.
- Redistribute Antigravity's OAuth client secret: it is read at runtime from your local Antigravity
  installation.

Product-level limits, such as never proxying model traffic, are the [non-goals](PRODUCT.md#non-goals).

## Reporting a vulnerability

Please use GitHub's private vulnerability reporting
(<https://github.com/shengyy/agyorbit/security/advisories/new>) rather than a public issue. Include the
version, platform, and steps to reproduce, without real tokens.
