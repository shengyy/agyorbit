# Contributing

Thanks for helping. This page is the human workflow; repository rules for coding agents live in
[`AGENTS.md`](AGENTS.md), and the code map in [`docs/architecture.md`](docs/architecture.md).

## Setup

- Install [Rust](https://rustup.rs) once per user through rustup. The root
  [`rust-toolchain.toml`](rust-toolchain.toml) registers this repository's version, components and
  installation profile with that shared installation. Rustup reuses the toolchain across repositories.
  Keep `CARGO_HOME` and `RUSTUP_HOME` at the workstation's user-level locations; do not create a separate
  Rust installation or download cache inside this repository.
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

### Rust caches

The workstation owns global Rust installation and cache settings. Use Cargo's built-in
[automatic global cache cleanup](https://doc.rust-lang.org/cargo/reference/config.html#global-caches)
with its defaults. It tracks downloaded dependencies; it does not clean this project's build output.

Keep `src-tauri/target/` between development and test runs so Cargo can reuse dependencies and
incremental compilation. When a deliberate build-cache reset is needed, stop active builds and use
the official [`cargo clean`](https://doc.rust-lang.org/cargo/commands/cargo-clean.html) command:

```bash
cargo clean --manifest-path src-tauri/Cargo.toml
```

This removes the build output and makes the next build cold. Run it on demand; build and deployment
commands must preserve the cache. Local installer and updater-package retention is covered in
[Local build](docs/release.md#local-build).

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

`main` is protected for everyone, maintainers included: no direct pushes, changes land through pull
requests only.

1. Branch from `main` and open a **draft** PR early. CI skips drafts.
2. When the checks above pass locally, mark the PR **ready for review**. CI then runs on macOS and
   Windows; the `gate` check must pass and the branch must contain the latest `main`.
3. Maintainers squash-merge.

- One focused change per PR, with a short imperative title.
- Add an entry under `[Unreleased]` in `CHANGELOG.md` for user-visible changes.
- Update both `README.md` and `README.zh-CN.md` when either changes.
- Never paste tokens, emails or log lines that contain them. Logs are designed not to include tokens;
  if you find one that does, that is a bug.
- UI changes: attach light and dark screenshots. `bun scripts/screenshot/render.ts` renders the real UI
  with fictional accounts; never post screenshots of real ones.

## Issues

Issues are the only to-do list; documents link to them instead of keeping their own.

- `bug`, `enhancement`: reports and requests. Check them against [PRODUCT.md](PRODUCT.md) first,
  especially its non-goals.
- `deferred`: known and accepted for now. The issue says why it waits and what would bring it back.
- `needs-decision`: waiting on a maintainer or user decision.
- `help wanted`, `good first issue`: good places to start.

Releases are cut by the maintainer (run end to end by their coding agent); see
[`docs/release.md`](docs/release.md).
