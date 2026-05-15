# claude-hud

A small always-on-top desktop HUD that shows how much of your Claude Pro / Max
quotas you've burned through — the 5-hour rolling window and the weekly
windows — refreshed live, callable via global hotkey.

Built with Tauri 2 + React.

## Status

Early prototype. Two display modes are switchable from the tray:

| Mode    | What it shows                                                  |
|---------|----------------------------------------------------------------|
| Minimal | Two bars (5h, week), reset countdowns, live/offline dot.       |
| Full    | Minimal + 24h sparkline + Opus / Sonnet weekly sub-bars.       |

Default hotkey: **Ctrl+Alt+H** toggles window visibility.

## How it gets the data

Calls the undocumented endpoint `GET https://api.anthropic.com/api/oauth/usage`
using the OAuth access token Claude Code stores at
`%USERPROFILE%\.claude\.credentials.json`. The token is read by the Rust
backend only — the frontend never sees it. Response shape:

```json
{
  "five_hour":  { "utilization": 20, "resets_at": "2026-05-15T22:50:00+00:00" },
  "seven_day":  { "utilization":  3, "resets_at": "2026-05-17T18:00:00+00:00" },
  "seven_day_opus":   null,
  "seven_day_sonnet": { "utilization": 2, "resets_at": "..." },
  "extra_usage": { "is_enabled": false, ... }
}
```

The endpoint is undocumented — Anthropic could change or remove it. The HUD
fails gracefully (shows a "sign in" or "offline" badge and keeps the last
known snapshot).

## Develop

Requirements: Rust 1.77+, Node 22+, pnpm.

```powershell
pnpm install
pnpm tauri dev
```

If `cargo` errors with `CRYPT_E_NO_REVOCATION_CHECK` on Windows, prefix the
command:

```powershell
$env:CARGO_HTTP_CHECK_REVOKE = "false"; pnpm tauri dev
```

## Build

```powershell
pnpm tauri build
```

Produces an MSI under `src-tauri/target/release/bundle/msi/`.

## Files

```
src-tauri/src/
  credentials.rs   reads ~/.claude/.credentials.json
  usage_client.rs  calls /api/oauth/usage
  poller.rs        interval task; emits usage://updated
  store.rs         settings + sample ring buffer; persists to %APPDATA%\claude-hud\state.json
  tray.rs          tray icon + mode menu
  hotkey.rs        global shortcut
  commands.rs      Tauri commands exposed to the UI
  lib.rs           wires it all together

src/
  modes/Minimal.tsx, modes/Full.tsx
  components/UsageBar.tsx, Sparkline.tsx
  hooks/useUsage.ts
  App.tsx, main.tsx, styles.css
```

## Known limits

- **No OAuth refresh flow.** When the access token expires the HUD shows a
  "sign in" state and asks you to run `claude` from a terminal to re-auth.
- **Windows only for now.** The Rust code is portable, but icons / bundling /
  hotkey defaults are tuned for Windows.
- **Single account.** No multi-account switching.
- **Endpoint is unofficial.** If Anthropic ships a documented one, swap it in
  inside `usage_client.rs`.

## Security

- Access token stays in the Rust process. It is never written to disk by this
  app (the credentials file is read each tick) and never sent to the frontend.
- `state.json` contains only usage samples + settings, no secrets.
- CSP on the webview allows no remote `connect-src` — only IPC.

## Roadmap

- OAuth token refresh.
- Agent-coordination data source (same plumbing as usage — different endpoint).
- macOS / Linux builds.
- Click-through mode (mouse passes through the HUD when not focused).
