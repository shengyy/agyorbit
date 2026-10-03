# Release

## Version

`src-tauri/Cargo.toml` is the only version source; the bundles, the app menu and the User-Agent read it.
Releases follow [SemVer](https://semver.org).

## Steps

1. Move `CHANGELOG.md`'s `[Unreleased]` entries under `## [X.Y.Z] - YYYY-MM-DD`.
2. Set `version` in `src-tauri/Cargo.toml`, run `cargo check` in `src-tauri` to update `Cargo.lock`.
3. Commit, then tag and push: `git tag vX.Y.Z && git push origin main vX.Y.Z`.
4. The `release` workflow builds a universal macOS `.dmg` and a Windows NSIS installer and attaches them
   to a draft GitHub release. Review the notes and publish.

## Signing

Builds are not signed with an Apple Developer ID or a Windows code-signing certificate yet.

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
`-setup.exe`, so a new release needs no site change. Preview locally:

```bash
mkdir -p /tmp/agyorbit-site/assets && cp -R site/. /tmp/agyorbit-site/ \
  && cp -R assets/brand assets/screenshots /tmp/agyorbit-site/assets/ \
  && python3 -m http.server 8000 --directory /tmp/agyorbit-site
```
