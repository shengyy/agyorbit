# Product

This is the single source for what AgyOrbit does, the rules it follows and what it will not do. How it
looks is in [docs/design.md](docs/design.md); what is verified today is in
[docs/status.md](docs/status.md).

## Purpose

AgyOrbit is for people who use the official Google Antigravity app with **more than one of their own
Google accounts**. It answers two questions from the menu bar (macOS) or tray (Windows): *how much quota
does each account have left?* and *switch Antigravity to that account, now.*

It is a switcher, not a client: Antigravity itself does all model work.

## Concepts

- **Account**: a Google account the user added to AgyOrbit. Identified by Google's stable account id;
  its refresh token lives in the system credential store.
- **Current account**: the account the Antigravity app is signed in with right now, read from
  Antigravity's own credential.
- **Quota group**: a set of models that share limits, as Google reports them. Today there are two:
  Gemini, and Claude (Claude Opus, Claude Sonnet, GPT-OSS).
- **Window**: each group has a 5-hour and a weekly limit. AgyOrbit shows the share **remaining** in each,
  and when it resets.
- **Switch**: making Antigravity use another account.

## Rules

### Accounts

- Accounts are added through Google's own sign-in page in the user's browser, never by asking for a
  password and never by signing out of Antigravity.
- Signing in to an account that is already listed renews its authorization.
- When Antigravity is signed in to an account AgyOrbit has not seen yet (for example after the user
  signed in inside the IDE), that account is added once, automatically.
- Removing an account forgets it in AgyOrbit only. It does not sign Antigravity out, and the account is
  not added back unless Antigravity signs in to it again.
- If Google revokes an account's authorization, the account stays listed and asks to be signed in again.

### Switching

- Anything that closes Antigravity asks first and names what will close: the app, its helpers and
  language server, and the `agy` CLI.
- A switch either completes or leaves the previous sign-in in place: nothing is touched until the target
  account's authorization is confirmed, and if writing or verifying the new sign-in fails, the previous
  one is restored.
- After a switch, Antigravity is opened again.
- Restart and Quit act on Antigravity only and also ask first.

### Quota

- Quotas refresh in the background every 5 minutes, and when the surface opens if they are older than a
  minute.
- The account in use is shown first. Another account is marked *Most left* only when it clearly beats the
  current one; Claude availability decides (the tighter of its 5-hour and weekly windows), Gemini breaks
  ties.
- A temporary error keeps the last good numbers instead of blanking them.

### App updates

- Check for new stable releases at startup and every 24 hours while running; the app menu can check
  immediately. A failed background check stays quiet.
- A new release appears as a small banner. The user can read its notes, update, or leave it for later.
- Download and installation start only after confirmation, while no account or process operation is
  running. The signed package and signed version must verify before installation.
- Updating restarts AgyOrbit only, preserving accounts and settings. Antigravity keeps running.

## Non-goals

- **No API, proxy or relay.** AgyOrbit never exposes accounts to other programs or forwards model
  traffic.
- **No sharing.** It is for one person's own accounts on their own computer.
- **No working around limits.** It does not switch accounts automatically when quota runs out; every
  switch is the user's decision.
- **No other clients.** It manages the official Antigravity app and its `agy` CLI, nothing else.
- **No platforms beyond macOS 26+ and Windows 10/11 x64** for now; there is no legacy-macOS fallback.

Feature requests that cross these lines are declined; anything else is weighed against keeping AgyOrbit
small.
