# Design

AgyOrbit should feel like part of the operating system: one glance to see every account's quota, one
confirmed click to switch.

## Surfaces

- **macOS**: a popover under the menu bar icon, Liquid Glass background, no Dock icon. It resizes to its
  content (up to 680 pt) and closes when focus leaves, like a menu. macOS 26 or later.
- **Windows**: a compact window opened from the tray icon. Closing it keeps AgyOrbit in the tray.
- Both run the same React UI; `platform-macos` / `platform-windows` on the root only change surface
  tokens and sizing.
- Left click toggles the surface. Right click opens a native menu (open, quit).

## Interaction rules

- Anything that closes Antigravity asks first, in a sheet inside the surface, and names what will close
  (app, language server, helpers, `agy` CLI). Enter confirms, Escape cancels.
- Long operations show their real progress (switch steps come from the backend) and cannot be dismissed
  half-way. Success lingers 1.6 s; errors stay until acknowledged.
- Adding an account happens in the browser. The surface shows a cancellable waiting sheet and comes back
  when Google redirects.
- Account actions (sign in again, copy email, remove) live in a native context menu: right-click a card
  or use its `⋯` button.
- Quotas refresh in the background every 5 minutes and when the surface opens if older than a minute.

## Account card

- The account in use is pinned first; the others keep the order they were added in.
- Avatar (Google photo, else initials), name, plan badge (`PRO` / `ULTRA` / `FREE`), email.
- `Current` marks the account Antigravity is signed in with; `Most left` marks the other account worth
  switching to (Claude availability decides, Gemini breaks ties, and only when it clearly beats the
  current one).
- One row per model group (Gemini, Claude) with the 5-hour and weekly windows side by side. The number
  is the share **remaining**. Colour changes only when it matters: amber below 35 %, red below 10 %.
- When a window is low, a footnote says when it recovers; hovering any meter shows its reset time.

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
