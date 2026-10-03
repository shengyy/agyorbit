# Release

## Version

`src-tauri/Cargo.toml` is the only version source; the bundles, the app menu and the User-Agent read it.
Releases follow [SemVer](https://semver.org).

## Steps

1. In a normal PR, move `CHANGELOG.md`'s `[Unreleased]` entries under `## [X.Y.Z] - YYYY-MM-DD`, set
   `version` in `src-tauri/Cargo.toml` and run `cargo check` in `src-tauri` to update `Cargo.lock`.
   Merge it.
2. Actions → **Release** → **Run workflow** on `main`, and enter `X.Y.Z`. The `authorize` job refuses to
   run from another branch, for a version that differs from `Cargo.toml`, or for one already released.
   The build then makes a universal macOS `.dmg` and a Windows NSIS installer from `main`'s current
   commit and attaches them to a draft release `vX.Y.Z`. Nothing is pushed from a local machine.
3. Check the draft: download the `.dmg`, confirm `codesign -dv` reports the identifier
   `io.github.shengyy.agyorbit` and `lipo -archs` lists both architectures. Then publish it; publishing
   creates the tag, and the website's download buttons follow the latest release automatically.

## Signing

Builds are not signed with an Apple Developer ID or a Windows code-signing certificate yet.

macOS bundles are ad-hoc signed during bundling (`bundle.macOS.signingIdentity: "-"`), which gives every
build the code identifier `io.github.shengyy.agyorbit`. Keep it: macOS keys the SMAppService login item
to the code identity, and the linker's default ad-hoc signature changes it on every build.

- macOS: Gatekeeper blocks the first launch of a downloaded build. Open it once with right-click → Open,
  or clear the quarantine flag: `xattr -dr com.apple.quarantine /Applications/AgyOrbit.app`.
- Windows: SmartScreen warns on first run; choose "More info" → "Run anyway".

Adding signing later only needs secrets in the workflow (`APPLE_CERTIFICATE`, `APPLE_SIGNING_IDENTITY`,
notarization credentials, or a Windows certificate) as described in Tauri's distribution guide.

## Local build

```bash
bun install
bun tauri build                 # host platform
bun tauri build --target universal-apple-darwin   # needs both Apple targets: rustup target add x86_64-apple-darwin
```

Artifacts land in `src-tauri/target/<target>/release/bundle/`.

## Website

`site/` is the landing page at <https://shengyy.github.io/agyorbit/>. The `pages` workflow deploys it on
every push to `main` that touches `site/` or `assets/`, copying `assets/brand` and `assets/screenshots`
next to it. The download buttons ask GitHub's API for the latest release and pick its `.dmg` and
`-setup.exe`, so a new release needs no site change. Sizes are in rem with a fluid root font size
(16px up to ~1280px wide, 22px on large displays), so the whole page scales up instead of leaving wide
margins; screenshots come in 2x and 3x through `srcset`. Preview locally:

```bash
mkdir -p /tmp/agyorbit-site/assets && cp -R site/. /tmp/agyorbit-site/ \
  && cp -R assets/brand assets/screenshots /tmp/agyorbit-site/assets/ \
  && python3 -m http.server 8000 --directory /tmp/agyorbit-site
```
