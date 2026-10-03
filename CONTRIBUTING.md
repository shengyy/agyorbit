# Contributing

Thanks for helping. This page is the human workflow; repository rules for coding agents live in
[`AGENTS.md`](AGENTS.md), and the code map in [`docs/architecture.md`](docs/architecture.md).

## Setup

- [Rust](https://rustup.rs) via rustup. `rust-toolchain.toml` pins the version; rustup installs it on
  first use.
- [Bun](https://bun.sh) for the frontend and the Tauri CLI.
- macOS: Xcode Command Line Tools. Windows: the MSVC build tools and WebView2 (preinstalled on Windows
  10/11). See Tauri's [prerequisites](https://tauri.app/start/prerequisites/).
- Antigravity installed and signed in, to try anything end to end.

```bash
bun install
bun tauri dev
```

The app appears in the menu bar (macOS) or the notification area (Windows). Running a second instance
just reveals the first, which is handy for opening the popover from a terminal.

## Checks

The same commands run in CI on macOS and Windows:

```bash
bun run check                       # Biome + TypeScript
bun run build                       # frontend bundle, needed before the Rust checks
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

### Verifying against a real Antigravity

```bash
cd src-tauri && cargo test system_tests -- --ignored --nocapture
```

This is read-only: it locates the installation, discovers the OAuth client, reads the signed-in account,
refreshes its token and fetches quota, printing nothing secret. If you verified something new (for
example on Windows or a new Antigravity version), update the table in
[`docs/antigravity-integration.md`](docs/antigravity-integration.md) in the same PR.

## Pull requests

- One focused change per PR, with a short imperative title.
- Add an entry under `[Unreleased]` in `CHANGELOG.md` for user-visible changes.
- Update both `README.md` and `README.zh-CN.md` when either changes.
- Never paste tokens, emails or log lines that contain them. Logs are designed not to include tokens;
  if you find one that does, that is a bug.
- UI changes: attach light and dark screenshots. `bun scripts/screenshot/render.ts` renders the real UI
  with fictional accounts; never post screenshots of real ones.

## Good first areas

- Windows verification on a real installation (credential layout, process names, install path).
- Signed and notarized release builds.
- More languages in `src/i18n.ts`.

Releases are cut by maintainers; see [`docs/release.md`](docs/release.md).
