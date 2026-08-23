# Software design

Covers *how the firmware is built*, as distinct from
[architecture.md](architecture.md) (system/network/hardware shape) and
[protocol.md](protocol.md) (wire format). See [ADR-0004](adr/0004-hexagonal-core-with-display-command-storage-ports.md),
[ADR-0005](adr/0005-googletest-native-testing.md), and
[ADR-0006](adr/0006-rust-for-lib-core.md) for the reasoning.

## Shape

Each firmware project splits into a hardware-free core and thin hardware
adapters. The Display's core is Rust ([ADR-0006](adr/0006-rust-for-lib-core.md)):

```
firmware/display/
  core/                # Rust crate — no hardware/OS deps (ADR-0006)
    src/
      state.rs              Side, SetResult, UndoSnapshot, MatchState
      command.rs             Command
      scoring.rs
      serve.rs
      undo.rs
      set_progression.rs
      match_logic.rs        pure apply(state, command) -> state
      ports.rs               Display / Storage traits
      application.rs         thin shell: apply() + storage.save() + display.render()
      lib.rs                 re-exports
    tests/
      state_test.rs
      command_test.rs
      scoring_test.rs
      serve_test.rs
      undo_test.rs
      set_progression_test.rs
      match_logic_test.rs
    Cargo.toml

  <adapters — implement the Display/Storage traits against real hardware
   (HUB75 panel, LittleFS) and wire ESP-NOW/HTTP commands into
   Application::handle(). Not built yet (slice 1 only covers the core);
   language/structure for this layer is a later decision.>

firmware/controller/
  lib/core/     # pure C++ — debounce/idle-timeout decision logic, Command construction
  src/          # Arduino adapters: ESP-NOW send, LED ack blink, deep sleep
  test/
  platformio.ini
```

No code is shared between the two projects' cores — they solve different
problems (match logic vs. debounce/idle-timing). (The Controller section
above reflects earlier grilling and hasn't been revisited since; treat it
as unconfirmed until that slice is actually grilled.)

## The core is a pure function, not a stateful object with side effects

`apply(state: &MatchState, cmd: &Command) -> MatchState` (Display's core) takes
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
traits, not any concrete implementation), so it's also unit-testable —
just with fake `Display`/`Storage` implementations instead of real
hardware, verifying e.g. "starting a Match calls storage.save() exactly
once" or "resuming from a Storage that returns a saved state re-renders it
on construction."

## Ports

```rust
pub trait Display {
    fn render(&mut self, state: &MatchState);
}

pub trait Storage {
    fn save(&mut self, state: &MatchState);
    fn load(&self) -> Option<MatchState>;
}
```

Real implementations (`Hub75Display`, `LittleFsStorage`) and fakes (an
in-memory `FakeStorage`, a spy `Display`) both just implement these two
traits.

## Command

A single value type, not a trait with a method per action — it composes
with a queue directly, and a scripted test fixture is just `Vec<Command>`.
A data-carrying enum, not a tagged struct: each variant holds only the
data that command needs, so there's no field left irrelevant depending on
which variant it is.

```rust
pub enum Command {
    StartMatch { name_left: String, name_right: String, best_of: u8 },
    Point { side: Side },
    Undo,
    SetServer { side: Side },
    Close,
}
```

Real Command sources (ESP-NOW receive callback, the phone's WebSocket/HTTP
handlers) construct a `Command` and push it into a single-writer queue so
`Application::handle()` is only ever called from one place — see ADR-0004's
"consequence" note: **the exact queue mechanism reconciling concurrent
sources is intentionally not designed yet**, only that they funnel through
one drain point rather than calling into `Application` directly from
whichever task they happen to run on.

## Testing

`cargo test`, run directly against the core crate on the host machine — no
ESP32 toolchain involved, fast enough to run on every save. Test coverage
for the Display core should include (not exhaustive — the actual `tests/`
files are the source of truth):

- A point for a Side increments that Side's score and recomputes `server`.
- Deuce (10-10) switches serve rotation to every point instead of every 2.
- Reaching 11 (or beyond, win-by-2) completes the Set, appends to history,
  resets the current score, and alternates `firstServerThisSet`.
- Winning a majority of `bestOf` Sets sets `decided`, but the Match stays
  In-Match (no auto-close).
- `Undo` reverses the last point, including reopening a just-completed Set
  (removing it from history, decrementing the Set win, restoring the
  pre-completion score) — and is a no-op on an empty stack.
- `SetServer` re-anchors `firstServerThisSet` so the current computed
  server becomes the requested Side, without disturbing the undo stack.
- `StartMatch` is rejected (state unchanged) when a Match is already
  active; accepted from Standby.
- `Close` returns to Standby and clears the undo stack regardless of
  whether the Match was decided.
