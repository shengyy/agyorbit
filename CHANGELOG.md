# Changelog

All notable changes to AgyOrbit are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

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

### Verified

- macOS 27 with Antigravity 2.19.1: OAuth client discovery, browser sign-in, quota and plan, and a
  switch between two Google AI Pro accounts (credential written, verified, Antigravity relaunched).
  Windows support compiles in CI but has not been verified on a real installation.
