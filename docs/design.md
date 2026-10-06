# Design

This page owns the app and website's visual presentation and interaction, icons and screenshots.
Product behavior lives in [PRODUCT.md](../PRODUCT.md); build, signing and website publishing live in
[release.md](release.md).

AgyOrbit should feel like part of the operating system: one glance to see every account's quota, one
confirmed click to switch.

## Surfaces

- **macOS**: a popover under the menu bar icon, Liquid Glass background, no Dock icon. It resizes to its
  content (up to 680 pt) and closes when focus leaves, like a menu. The WebView lives inside the native
  Liquid Glass container through `window-vibrancy`’s `content_view` API; that container clips all page
  fills and confirmation backdrops to the same 18 pt corners. The extra system window shadow is
  disabled. macOS 26 or later.
- **Windows**: a compact window opened from the tray icon. Closing it keeps AgyOrbit in the tray.
- Both run the same React UI; `platform-macos` / `platform-windows` on the root only change surface
  tokens and sizing.
- Left click toggles the surface. Right click opens a native menu (open, quit).
- Open at Login shows AgyOrbit by name and icon in macOS System Settings; on Windows it starts hidden
  in the tray. Registration is described in [architecture.md](architecture.md).

## Interaction

What happens and when is defined in [PRODUCT.md](../PRODUCT.md); this is how it is presented.

- Confirmations are sheets that slide up inside the surface, never separate windows. Enter confirms,
  Escape cancels, and a confirmation names what it will close.
- Long operations show their real progress (switch steps come from the backend) and cannot be dismissed
  half-way. Success lingers 1.6 s; errors stay until acknowledged.
- While an account signs in, the surface shows a cancellable waiting sheet and comes back when Google
  redirects.
- Updates appear in a compact banner under the header. Clicking opens the existing sheet style with
  current and new versions, scrollable release notes, Later and Update & Restart. Downloads show real
  progress, including an indeterminate bar when the server gives no size; verification and installation
  have separate steps. The app menu also offers Check for Updates.
- Account actions (sign in again, copy email, remove) live in a native context menu: right-click a card
  or use its `⋯` button. App actions live in the header's `⋯` menu.

## Account card

- Avatar (Google photo, else initials), name, plan badge (`PRO` / `ULTRA` / `FREE`), email.
- Tags follow the name and plan badge: `Current` for the account in use, `Most left` for the account
  worth switching to.
- One row per quota group (Gemini, Claude) with the 5-hour and weekly windows side by side, each a meter
  and the share remaining. Colour changes only when it matters: amber below 35 %, red below 10 %.
- When a window is low, a footnote says when it recovers; hovering any meter shows its reset time.
- The Switch button appears on hover or keyboard focus in reserved space beside the email, so it never
  overlaps the name, plan or tags.

## Visual tokens

Defined once in `src/styles/tokens.css`: text and fill opacities, accent, meter colours, card and sheet
surfaces, light and dark. Components use tokens only. Type is the system font at 13 px (11–11.5 px for
secondary text) with tabular numerals for percentages.

## Icon

A planet (Antigravity) with accounts in orbit; the glowing satellite is the active account. The app
icon is `assets/brand/app-icon.svg`; the menu bar template (black + alpha, 18 pt high) is
`assets/brand/tray-template.svg`. AgyOrbit does not reuse Antigravity's or Google's marks.

Regenerate the bundled icons after editing the SVGs:

```bash
rsvg-convert -w 1024 -h 1024 assets/brand/app-icon.svg -o /tmp/agyorbit-icon.png
bun tauri icon /tmp/agyorbit-icon.png -o src-tauri/icons
rm -rf src-tauri/icons/android src-tauri/icons/ios src-tauri/icons/Square*Logo.png src-tauri/icons/StoreLogo.png src-tauri/icons/64x64.png
rsvg-convert -h 36 assets/brand/tray-template.svg -o src-tauri/icons/tray-template.png
```

## Screenshots

README and website screenshots are rendered from the real UI with fictional accounts, so they never
show anyone's email, and always match the current design:

```bash
bun scripts/screenshot/render.ts   # needs Google Chrome and ImageMagick
```

It builds the frontend, replaces Tauri's IPC with `scripts/screenshot/mock-tauri.js` (demo data and
scenes), stages the popover under a menu bar with `frame.css`, renders at 3x and writes
`assets/screenshots/{hero,flow}-{light,dark}@3x.png` plus 2x copies without the suffix. The README uses
the 2x images; the website offers both through `srcset`. Never commit screenshots of real accounts.

## Website

`site/style.css` uses rem sizes with a fluid root font size (16px up to ~1280px wide, 22px on large
displays), so the whole page scales up instead of leaving wide margins. Screenshot generation and
resolutions are defined above; `site/index.html` selects them through `srcset`. Publishing, download
button behavior and local preview are in [release.md](release.md#website).
