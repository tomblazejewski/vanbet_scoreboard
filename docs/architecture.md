# Architecture

See [`CONTEXT.md`](../CONTEXT.md) for vocabulary (Set, Match, Side, Server,
Controller, Display) and [`docs/adr/`](adr/) for the reasoning behind the
decisions marked ADR-000x below.

## Components

```
┌─────────────────────┐        ESP-NOW (2.4GHz,          ┌───────────────────────┐
│   Controller           │        pinned WiFi channel,        │     Display               │
│   Left / Right / Undo │ ─────  see ADR-0001) ───────────▶ │     ESP32 + 64x64 P4 HUB75│
│   3 buttons, battery   │                                    │                            │
└─────────────────────┘                                    │  - WiFi station (office)  │
                                                              │  - mDNS: scoreboard.local │
┌─────────────────────┐        HTTP + WebSocket,            │  - Async web server + WS  │
│   Phone browser         │ ◀────  open on office WiFi ────▶ │  - Match state + undo stack│
│   (any device, no auth,│        no auth, any # of phones   │  - LittleFS persistence   │
│    no locking)          │                                    └───────────────────────┘
└─────────────────────┘
```

- **Display** is the sole source of truth. Controller and phone only ever
  send discrete commands; neither holds state.
- **Controller** has exactly three buttons — Left point, Right point, Undo —
  mapped to physical position, not player identity (ADR-0003).
- **Phone** is a thin client: renders whatever state the Display last pushed
  over the WebSocket, sends the same commands plus the two setup actions
  that don't belong on a 3-button remote (start Match, set names/bestOf).
- No authentication, no session/locking model on either control surface —
  any device on the office WiFi can issue any command, same as anyone
  standing at the table can press the Controller.

## Networking

The Display joins the office WiFi as a station (not AP mode) so multiple
phones can reach it via `scoreboard.local`, and to keep OTA/NTP available.
The Controller reaches it over ESP-NOW, which requires both radios on the
same WiFi channel — see [ADR-0001](adr/0001-espnow-controller-link-pinned-channel.md)
for why that channel is pinned on the router rather than discovered
dynamically, and what breaks if the router's channel ever changes.

ESP-NOW peer traffic is encrypted with a pre-shared key baked into both
firmwares at build time (`secrets.h`, gitignored) — low stakes for an office
scoreboard, but cheap insurance against a neighboring desk's ESP-NOW gadget
sending stray commands.

## Match lifecycle

Two real states at the persistence/lifecycle level — "decided" (one Side has
won a majority of Sets) is a computed property *within* `MATCH_ACTIVE`, not a
separate phase, because nothing about how endpoints behave changes when a
Match becomes decided except that new points stop making sense to send.
Closing is always explicit, even after a natural win — see
[ADR-0002](adr/0002-persist-and-resume-explicit-close.md).

```
   NO_MATCH ──── POST /api/match {nameLeft, nameRight, bestOf} ────▶ MATCH_ACTIVE
      ▲                                                                    │
      └──────────────────── POST /api/close ─────────────────────────────┘
```

`POST /api/match` is rejected with an error if a Match is already active —
starting a new one always requires closing the previous one first, so a
Match is never silently discarded.

State (names, bestOf, current score, Set history, server, undo stack) is
written to LittleFS as JSON after every mutating command. On boot, if saved
state exists the Display resumes `MATCH_ACTIVE` immediately with no
staleness check — see ADR-0002.

## Data model (display firmware)

Shown in Rust ([ADR-0006](adr/0006-rust-for-lib-core.md); this is the
Display's state-manipulation core specifically, not the whole firmware —
see [software-design.md](software-design.md)):

```rust
const POINTS_TO_WIN: u8 = 11;   // fixed, not configurable — see match-rules.md
const MAX_SETS: u8 = 11;        // capped maximum bestOf (odd, ≤ 11)
const MAX_UNDO: u16 = 200;      // generous headroom for a full match

pub enum Side { Left, Right }

pub struct SetResult { pub score_left: u8, pub score_right: u8 }

// Pushed before every Point or SetServer is applied; popped + restored on
// undo. Both mutate server-rotation state (and Point may also mutate
// score/history/setsWon on a Set completion), and undo covers reversing
// either kind of mistake.
pub struct UndoSnapshot {
    pub score_left: u8,
    pub score_right: u8,
    pub sets_won_left: u8,
    pub sets_won_right: u8,
    pub server: Side,
    pub first_server_this_set: Side,
    pub history_count: u8,
}

pub struct MatchState {
    pub active: bool,                 // false == Standby
    pub name_left: [u8; 64],          // fixed buffer, not String — this is the
    pub name_right: [u8; 64],         // persisted state, unlike Command's names
                                       // (64 bytes: UTF-8 headroom, not just ASCII)
    pub best_of: u8,                  // odd, capped at 11 (enforced at REST boundary)
    pub sets_won_left: u8,
    pub sets_won_right: u8,
    pub score_left: u8,               // current Set in progress
    pub score_right: u8,
    pub server: Side,
    pub first_server_this_set: Side,  // anchor used to compute server from point count
    pub history: [SetResult; MAX_SETS as usize],
    pub history_count: u8,
    pub undo_stack: [UndoSnapshot; MAX_UNDO as usize],
    pub undo_count: u16,
}
```

## Serve computation

`server` is never incremented directly — it's derived from
`firstServerThisSet` and how many points have been played this Set, per the
rotation rules in [match-rules.md](match-rules.md). `SET_SERVER {side}`
(the dedicated endpoint, used both for the initial "who serves first" call
and any later correction) works by solving for the `firstServerThisSet`
value that makes the *current* computed server equal the requested side —
so auto-rotation continues correctly from wherever the correction happened,
rather than just overwriting `server` and going stale on the next point.

## Rendering

64x64 at P4 pitch. Planned layout (v1, revisit once a panel is in hand):

```
┌────────────────────────────────┐
│ ●LEFT NAME       9   RIGHT NAME │  <- names (truncated) + serve dot on the current Side
│                                  │
│      7                5          │  <- big current-Set score
│                                  │
│  11-7  9-11  •••                │  <- past Set scores, small font, bottom row
└────────────────────────────────┘
```

Big score digits need a custom bitmap font (default Adafruit GFX font is too
small to read across an office) — deferred to a v1 follow-up once real
hardware is in hand to tune against.

## Firmware stack

- [`ESP32-HUB75-MatrixPanel-I2S-DMA`](https://github.com/mrfaptastic/ESP32-HUB75-MatrixPanel-I2S-DMA) — panel driver (DMA-driven, doesn't busy-loop the CPU, leaves room for WiFi/ESP-NOW).
- `ESPAsyncWebServer` + `AsyncTCP` — HTTP + WebSocket server for the phone UI.
- `ArduinoJson` — state serialization (persistence + WebSocket payloads).
- Arduino-ESP32 core's `esp_now.h` — Controller link.
- `ESPmDNS` — `scoreboard.local`.
- `ArduinoOTA` — flash updates over WiFi once the panel is mounted.

## Explicitly out of scope for v1

- Player roster / saved names — every Match starts from a fresh `POST`.
- Auth / access control on any endpoint.
- Per-player remotes or any player-aware Controller behavior — scoring is
  strictly Side-indexed (ADR-0003).
- Configurable Set format (points-to-win, win-by-2) — fixed at 11/win-by-2.
- Auto-clearing a finished Match — always explicit close (ADR-0002).
