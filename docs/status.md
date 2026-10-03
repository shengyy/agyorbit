# Status

This page owns platform support and the scope of feature verification. Antigravity's external
contracts and their verified versions live in [antigravity-integration.md](antigravity-integration.md).
The version itself lives in `src-tauri/Cargo.toml`, published
builds on [Releases](https://github.com/shengyy/agyorbit/releases), and the change history in
[CHANGELOG.md](../CHANGELOG.md). Update this page in the same PR as anything that changes what has been
verified.

## Platforms

| | macOS 26+ | Windows 10/11 x64 |
|---|---|---|
| Builds and CI checks | Yes | Yes |
| Installer | Universal `.dmg` (Apple silicon and Intel), ad-hoc signed | NSIS `-setup.exe`, unsigned |
| Verified on a real installation | Yes: macOS 27, Antigravity 2.19.1 | **Not verified** ([#3](https://github.com/shengyy/agyorbit/issues/3)) |

## Verified on macOS (0.1.2)

- OAuth client discovery from the installed Antigravity, browser sign-in, adopting the IDE's own
  sign-in.
- Quota and plan for seven Google AI Pro accounts.
- Repeated switches between accounts, with Antigravity reopening on the new account.
- Every menu item, Open at Login through SMAppService, and the login item surviving an update.

Each external fact these rely on, and the Antigravity version it was checked against, is listed in
[antigravity-integration.md](antigravity-integration.md).

## Known limitations

Accepted limitations are tracked as issues labeled
[`deferred`](https://github.com/shengyy/agyorbit/issues?q=is%3Aissue+is%3Aopen+label%3Adeferred), each
saying why it waits and what would bring it back. They are not repeated here.
