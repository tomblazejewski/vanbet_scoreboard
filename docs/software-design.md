# Software design

Covers *how the firmware is built*, as distinct from
[architecture.md](architecture.md) (system/network/hardware shape) and
[protocol.md](protocol.md) (wire format). See [ADR-0004](adr/0004-hexagonal-core-with-display-command-storage-ports.md)
and [ADR-0005](adr/0005-googletest-native-testing.md) for the reasoning.

## Shape

Each firmware project (`firmware/display/`, `firmware/controller/`) is split
into a hardware-free core and thin hardware adapters:

```
firmware/display/
  lib/core/     # pure C++ — no Arduino.h, no ESP-IDF, no hardware headers
    command.h      Side, Command, MatchState, SetResult
    match_logic.h/.cpp   pure apply(state, command) -> state
    ports.h         abstract Display / Storage interfaces
    application.h/.cpp   thin shell: apply() + storage.save() + display.render()
  src/          # Arduino adapters implementing the ports, + setup()/loop() wiring
  test/         # GoogleTest, links only against lib/core
  platformio.ini   # env:esp32dev (device) + env:native (tests)

firmware/controller/
  lib/core/     # pure C++ — debounce/idle-timeout decision logic, Command construction
  src/          # Arduino adapters: ESP-NOW send, LED ack blink, deep sleep
  test/
  platformio.ini
```

No code is shared between the two projects' cores — they solve different
problems (match logic vs. debounce/idle-timing).

## The core is a pure function, not a stateful object with side effects

`apply(const MatchState&, const Command&) -> MatchState` (Display's core) takes
a state and a command and returns the resulting state — no I/O, no globals, no
port calls. This is what makes it trivial to test: construct a `MatchState`,
apply a `Command`, assert on the result — no fakes/mocks needed at all for
this layer.

The one place ports get called is `Application`, a thin shell that:
1. holds the current `MatchState` and references to a `Display` and a `Storage`,
2. on construction, calls `storage.load()` to resume (or starts at `NO_MATCH`),
3. on `handle(Command)`, calls `apply()`, stores the result, then calls
   `storage.save(state)` and `display.render(state)`.

`Application` is still hardware-free (it only knows about the abstract port
interfaces, not any concrete implementation), so it's also unit-testable —
just with `MockDisplay`/`MockStorage` (gmock) instead of real hardware,
verifying e.g. "starting a Match calls storage.save() exactly once" or
"resuming from a Storage that returns a saved state re-renders it on
construction."

## Ports

```cpp
class Display {
public:
  virtual void render(const MatchState&) = 0;
  virtual ~Display() = default;
};

class Storage {
public:
  virtual void save(const MatchState&) = 0;
  virtual std::optional<MatchState> load() = 0;
  virtual ~Storage() = default;
};
```

Real implementations (`Hub75Display`, `LittleFsStorage`) and fakes
(`MockDisplay`, `MockStorage`, or a simple in-memory `FakeStorage`) both just
implement these two methods.

## Command

A single tagged value type, not an interface with a method per action — see
the rationale in the grilling transcript: it composes with a queue directly,
and a scripted test fixture is just `std::vector<Command>`.

```cpp
enum class CommandType { START_MATCH, POINT, UNDO, SET_SERVER, CLOSE };

struct Command {
  CommandType type;
  Side side;                 // POINT, SET_SERVER
  std::string nameLeft, nameRight;  // START_MATCH
  uint8_t bestOf;             // START_MATCH
};
```

Real Command sources (ESP-NOW receive callback, the phone's WebSocket/HTTP
handlers) construct a `Command` and push it into a single-writer queue so
`Application::handle()` is only ever called from one place — see ADR-0004's
"consequence" note: **the exact queue mechanism is intentionally not designed
yet**, only that concurrent sources funnel through one drain point rather
than calling into `Application` directly from whichever FreeRTOS task they
happen to run on.

## Testing

GoogleTest, run via `pio test -e native` (see ADR-0005) — a `native`
PlatformIO environment with no ESP32 toolchain involved, fast enough to run
on every save. Test coverage for the Display core should include (not
exhaustive — the actual `test/` files are the source of truth):

- A point for a Side increments that Side's score and recomputes `server`.
- Deuce (10-10) switches serve rotation to every point instead of every 2.
- Reaching 11 (or beyond, win-by-2) completes the Set, appends to history,
  resets the current score, and alternates `firstServerThisSet`.
- Winning a majority of `bestOf` Sets sets `decided`, but the Match stays
  `MATCH_ACTIVE` (no auto-close).
- `UNDO` reverses the last point, including reopening a just-completed Set
  (removing it from history, decrementing the Set win, restoring the
  pre-completion score) — and is a no-op on an empty stack.
- `SET_SERVER` re-anchors `firstServerThisSet` so the current computed
  server becomes the requested Side, without disturbing the undo stack.
- `START_MATCH` is rejected (state unchanged) when a Match is already
  active; accepted from `NO_MATCH`.
- `CLOSE` returns to `NO_MATCH` and clears the undo stack regardless of
  whether the Match was decided.
