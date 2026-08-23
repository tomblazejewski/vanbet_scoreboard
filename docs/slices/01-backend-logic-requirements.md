# Slice 1: Backend logic — requirements

Grilled and resolved (see git history for the session). Terms below match
[`CONTEXT.md`](../../CONTEXT.md).

## Goal

The pure Match/Set/serve/undo logic — `lib/core`'s `apply()` — with nothing
else attached. No Display, no Storage, no networking, no Arduino. A
function that takes a state and a command and returns a state, plus tests
proving it's right: **every case should be testable by performing an
action on a state and checking what state comes out.**

## Function shape

```cpp
MatchState apply(const MatchState& state, const Command& cmd);
```

Always returns a plain `MatchState` — no accepted/rejected side-channel.
A command that doesn't apply in the current state (there's nothing to
`Undo`, etc.) just returns the input state unchanged, same as any other
identity transform. Whether a caller should have been allowed to send that
command in the first place is a concern for a layer above this slice (the
REST boundary in slice 3), not something the pure core signals.

## In scope

- Data shapes: `Side`, `SetResult`, `UndoSnapshot`, `MatchState`, `Command`.
- Commands: `Start-match`, `Point`, `Undo`, `Set-server`, `Close`,
  **`Unlock`**.
- The rules in [match-rules.md](../match-rules.md): Set win at
  11/win-by-2, Deuce serve rotation, Set/Match progression, Set history,
  the undo stack (including reopening a completed Set), `Set-server`'s
  re-anchoring math, `Point` freezing once decided (except via `Undo` or
  `Unlock`).
- GoogleTest test suite under `test/`, run via PlatformIO's `env:native`
  (ADR-0005).

## Explicitly out of scope

- The `Display` and `Storage` ports, and the `Application` shell that
  calls them — this slice produces `apply()` only; wiring it to real or
  fake ports is a later integration point (touches slices 2 and 3).
- The Command queue / concurrency story (ADR-0004 leaves this open on
  purpose).
- **All input validation** — `bestOf` being odd and capped at 11, name
  length — is enforced at the REST boundary (slice 3). This slice's
  `apply()` may assume a `Start-match` command it receives already
  satisfies these constraints; it does not need to handle or test
  malformed input.
- Anything hardware- or network-facing.

## Still open (deliberately deferred, not blocking this slice)

- **`Unlock`'s exact state representation.** Candidate approaches
  discussed: (a) a single flag doing double duty — "concluded by standard
  rules" *is* what blocks `Point`, and `Unlock` clears it outright, so
  post-unlock state looks as if the Match never concluded; or (b) two
  separate things — a permanent record that the Match *did* conclude by
  the rules at some point, plus an independent "currently frozen" switch
  that `Unlock` flips off. Not resolved — pick one when actually
  implementing, informed by whatever slice 2 (Display protocol) turns out
  to need to render.

## Deliverables

- `firmware/display/lib/core/command.h` — data types.
- `firmware/display/lib/core/match_logic.h` / `.cpp` — pure `apply()`.
- `firmware/display/test/test_match_logic.cpp` — GoogleTest cases.
- `firmware/display/platformio.ini` — `env:native` added (device env can
  wait for a later slice; it's not needed to run these tests).

## Constants

- `POINTS_TO_WIN = 11` (fixed — see match-rules.md).
- `MAX_SETS = 11` — sized to the capped maximum `bestOf` (odd, ≤ 11), not
  the original "3/5/7" framing.
- `MAX_UNDO` — an implementation constant, not a domain decision; picked
  generously (sized for the higher Set cap) and documented in code, with
  the ceiling behaving as "oldest undo capability quietly stops being
  available" if ever hit, not an error.
- `NAME_LEN = 16` — length itself is a REST-boundary validation concern
  (see "Explicitly out of scope"); this slice just needs a buffer big
  enough for whatever the REST layer already validated.

## Test scenarios to cover

1. A `Point` for a Side increments that Side's score by 1 and leaves the
   other untouched.
2. Serve alternates every 2 points in normal play.
3. Reaching 10-10 (Deuce) switches serve rotation to every 1 point.
4. Reaching 11 with a 2+ point lead completes the Set: appended to
   history, winner's Set-win count incremented, both scores reset to 0,
   `firstServerThisSet` (and therefore `server`) alternates from the Set
   just completed.
5. Reaching 11 with only a 1-point lead does *not* complete the Set (play
   continues).
6. Winning a majority of `bestOf` Sets marks the Match decided, but it
   stays In-Match (no auto-`Close` — ADR-0002).
7. **Once decided, `Point` leaves state unchanged** (the freeze).
8. **`Undo` still works once decided** — reverses the match-ending point,
   including un-deciding the Match if that point was what decided it.
9. `Undo` reverses the immediately preceding `Point` in the normal
   (not-yet-decided) case too (score, server, and — if that point had
   just completed a Set — historyCount/Set-win count all revert).
10. Repeated `Undo` walks back multiple points in sequence, including
    across a Set boundary (reopening a just-completed Set).
11. `Undo` on an empty stack is a no-op (state unchanged).
12. `Set-server {side}` sets the currently-computed server to `side`
    without touching the undo stack, and a subsequent `Point` continues
    auto-rotation correctly from that corrected point (not from square
    one).
13. `Start-match` from Standby succeeds: sets names/bestOf, resets all
    scores/history/undo, transitions to In-Match.
14. `Start-match` while already In-Match is a no-op (state unchanged) —
    matches the general "doesn't apply here" identity-transform rule,
    same as any other command that doesn't apply.
15. `Close` returns to Standby and clears the undo stack, regardless of
    whether the Match was decided.
16. `Unlock` on a decided Match allows a subsequent `Point` to change
    state again (exact intermediate state shape per the open
    representation question above — pin down the specific assertions once
    that's decided during implementation).
