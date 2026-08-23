# Wire protocol

Two independent links into the Display. Both carry Side-indexed commands
(never player-indexed — see [ADR-0003](adr/0003-side-indexed-by-controller-position.md))
against the same shared Match state.

## 1. Controller → Display (ESP-NOW)

Fixed-size packed struct, sent as an ESP-NOW encrypted peer message
(pre-shared LMK baked into both firmwares at build time via `secrets.h`,
never committed — see `firmware/*/src/secrets.example.h`). Requires both
devices on the same WiFi channel — see
[ADR-0001](adr/0001-espnow-controller-link-pinned-channel.md).

```cpp
#pragma pack(push, 1)
struct RemoteMsg {
  uint8_t  protoVersion;   // bump on breaking changes, Display ignores unknown versions
  uint8_t  seq;            // rolling counter, lets Display dedupe retransmits
  uint8_t  command;        // see RemoteCommand enum below
};
#pragma pack(pop)

enum RemoteCommand : uint8_t {
  CMD_POINT_LEFT  = 1,
  CMD_POINT_RIGHT = 2,
  CMD_UNDO        = 3,
  CMD_PING        = 4,   // heartbeat/battery-check, not a scoring action
};
```

The Controller has exactly three buttons: Left point, Right point, Undo.
There is no Controller-side "set server" button — that's a phone/API-only
action (see below), since it needs to name a Side explicitly rather than
just "the thing this button means."

Display acks every message with a tiny `{seq, ok}` unicast reply so the
Controller can flash an LED/vibrate on success and retry on silence.

## 2. Phone ↔ Display (HTTP + WebSocket)

- `GET /` — serves the control web app (static files from LittleFS).
- `WS /ws` — on connect, Display immediately pushes full current state.
  After that, Display pushes full state again after every mutating command,
  to every connected client — no diffing, no per-client targeting, since
  there's no locking/ownership model (any phone can control, all phones see
  the same thing).

State push (Display → phone), JSON:

```json
{
  "active": true,
  "nameLeft": "ALEX", "nameRight": "JORDAN",
  "bestOf": 5,
  "setsWonLeft": 1, "setsWonRight": 0,
  "scoreLeft": 7, "scoreRight": 5,
  "server": "left",
  "history": [ { "scoreLeft": 11, "scoreRight": 7 } ],
  "decided": false,
  "canUndo": true
}
```

`{"active": false}` (all other fields omitted) represents `NO_MATCH`.

### Endpoints (also reachable as plain HTTP for scripting/testing — same
handlers as the WebSocket commands, just without a persistent connection):

| Method | Path | Body | Behavior |
|---|---|---|---|
| `POST` | `/api/match` | `{"nameLeft","nameRight","bestOf"}` | Starts a Match. **Errors if one is already active** — must `close` first. |
| `POST` | `/api/point` | `{"side": "left"\|"right"}` | Scores a point for that Side. Pushes an undo snapshot first. |
| `POST` | `/api/undo` | — | Pops and restores the last undo snapshot. No-op if the stack is empty. |
| `POST` | `/api/server` | `{"side": "left"\|"right"}` | Sets the current server to that Side — used for the initial "who serves first" call and any later correction. Not pushed onto the undo stack. |
| `POST` | `/api/close` | — | Ends the active Match (whether decided or not) and returns to `NO_MATCH`. Clears the undo stack. |
| `GET` | `/api/state` | — | Returns the current state JSON (same shape as the WebSocket push) — handy for `curl`/debugging without opening a WebSocket. |

No authentication on any of these — see architecture.md's "explicitly out of
scope" list.

## Versioning

Both the ESP-NOW struct and the WebSocket/HTTP JSON carry enough redundancy
(protoVersion byte / tolerant JSON parsing with `ArduinoJson`) that Display
and Controller firmware don't have to be flashed in lockstep during
development.
