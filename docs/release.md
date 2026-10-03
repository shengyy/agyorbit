# Release

## Version

`src-tauri/Cargo.toml` is the only version source; the bundles, the app menu and the User-Agent read it.
Releases follow [SemVer](https://semver.org).

## Steps

Releases are run end to end by the maintainer's coding agent with `gh`; nobody needs to open the
browser. The maintainer decides *when*; the agent does the rest.

1. **Prepare in a normal PR.** Make sure [status.md](status.md) matches what this version was verified
   on, move `CHANGELOG.md`'s `[Unreleased]` entries under `## [X.Y.Z] - YYYY-MM-DD`, set `version` in
   `src-tauri/Cargo.toml` and run `cargo check` in `src-tauri` to update `Cargo.lock`. Merge it through
   the usual draft → ready → `gate` flow.
2. **Build the draft.**

   ```bash
   gh workflow run release.yml --ref main -f version=X.Y.Z
   gh run watch "$(gh run list --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId')" --exit-status
   ```

   The `authorize` job refuses another branch, a version that differs from `Cargo.toml`, or one already
   released. The build makes a universal macOS `.dmg` and a Windows NSIS installer from `main`'s current
   commit and attaches them to a draft release `vX.Y.Z`. Nothing is pushed from a local machine.
3. **Check the draft.**

   ```bash
   gh release download vX.Y.Z --pattern '*.dmg' --dir <tmp>
   gh release view vX.Y.Z --json assets --jq '.assets[] | "\(.name) \(.digest)"'   # compare with shasum -a 256
   ```

   Mount the `.dmg` and confirm `codesign -dv` reports the identifier `io.github.shengyy.agyorbit`,
   `lipo -archs` lists `x86_64 arm64`, and `CFBundleShortVersionString` is `X.Y.Z`; the Windows
   `-setup.exe` must be attached too.
4. **Publish.** Write the release notes from the version's `CHANGELOG.md` section, then:

   ```bash
   gh release edit vX.Y.Z --draft=false --latest --title "AgyOrbit X.Y.Z" --notes-file <notes.md>
   gh api repos/shengyy/agyorbit/releases/latest --jq .tag_name   # must print vX.Y.Z
   ```

   Publishing creates the tag; the website's download buttons follow the latest release automatically.

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
