# Changelog

All notable changes to AgyOrbit are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.3] - 2026-10-04

### Fixed

- The Switch button no longer overlaps the Most left tag. Account tags follow the name and plan badge,
  and the button has its own space beside the email.

## [0.1.2] - 2026-10-03

### Fixed

- Menu items did nothing when clicked (Show Logs, GitHub, Quit, and the account menu's Sign In Again,
  Copy Email and Remove). Tauri 2.12 drops items declared inline in `Menu.new` right after building the
  menu, which removes their click handlers; every item is now created as its own resource.
- Copy Email writes through the system clipboard; the web clipboard API refuses writes that do not come
  from a click inside the page.
- Open at Login survives updates on macOS. Builds are now ad-hoc signed with the bundle identifier, so
  every version has the same code identity, which is what macOS ties the login item to. Turn it on once
  more after updating from 0.1.1.

### Added

- *AgyOrbit Website* in the `⋯` menu.

## [0.1.1] - 2026-10-03

### Changed

- Open at Login registers AgyOrbit as a login item through SMAppService on macOS, so System Settings
  lists it by name and icon instead of as a background item from an unidentified developer. If you
  turned it on in 0.1.0, delete `~/Library/LaunchAgents/AgyOrbit.plist` and turn it on again. Windows
  keeps the per-user Run key.
- Website: the page scales with large displays instead of leaving wide margins, with sharper
  screenshots.

## [0.1.0] - 2026-10-03

### Added

- Menu bar popover on macOS 26+ (Liquid Glass) and a tray window on Windows.
- Every account's Gemini and Claude quota, 5-hour and weekly windows, with reset times and a
  "most left" hint for the account worth switching to.
- Browser sign-in (OAuth with PKCE on a loopback port) to add or re-authorize accounts without signing
  out of Antigravity; the account Antigravity is already signed in with is adopted automatically.
- Confirmed one-click switching: stops Antigravity and the `agy` CLI, writes the credential to the
  keychain and its fallback file, verifies it, relaunches, and restores the previous sign-in on failure.
- Restart and quit Antigravity, open at login, logs.
- English and Simplified Chinese UI.
- Website at https://shengyy.github.io/agyorbit/ with download links to the latest release.

### Verified

- macOS 27 with Antigravity 2.19.1: OAuth client discovery, browser sign-in, quota and plan, and a
  switch between two Google AI Pro accounts (credential written, verified, Antigravity relaunched).
  Windows support compiles in CI but has not been verified on a real installation.
