# Slice 1: Backend logic — requirements

Grilled and resolved (see git history for the session). Terms below match
[`CONTEXT.md`](../../CONTEXT.md).

## Goal

The pure Match/Set/serve/undo logic — the state-manipulation core's
`apply()` — with nothing else attached. No Display, no Storage, no
networking, no hardware framework (Arduino, ESP-IDF, or otherwise). A
function that takes a state and a command and returns a state, plus tests
proving it's right: **every case should be testable by performing an
action on a state and checking what state comes out.**

## Function shape

```
apply(state, command) -> state
```

A pure function: given the current state and one command, returns the
resulting state. No accepted/rejected side-channel — always returns a
plain state value. A command that doesn't apply in the current state
(there's nothing to `Undo`, etc.) just returns the input state unchanged,
same as any other identity transform. Whether a caller should have been
allowed to send that command in the first place is a concern for a layer
above this slice (the REST boundary in slice 3), not something the pure
core signals.

(For the concrete signature and file layout, see
[software-design.md](../software-design.md) and whichever ADR under
[docs/adr/](../adr/) records the implementation language — per
`CLAUDE.md`'s "Slicing conventions", that choice lives there, not here.)

## In scope

- Data shapes: `Side`, `SetResult`, `UndoSnapshot`, `MatchState`, `Command`.
- Commands: `Start-match`, `Point`, `Undo`, `Set-server`, `Close`.
- The rules in [match-rules.md](../match-rules.md): Set win at
  11/win-by-2, Deuce serve rotation, Set/Match progression, Set history,
  the undo stack (both `Point` and `Set-server` push onto it — reopening a
  completed Set and reversing a server correction are both `Undo`'s job),
  `Set-server`'s re-anchoring math, `Point` and `Set-server` both freezing
  once decided (except via `Undo`), and `Point`/`Undo`/`Set-server` all
  being no-ops while there's no Match in progress (only `Start-match` does
  anything from Standby).
- An automated test suite, runnable on the host machine with no hardware
  or cross-compilation involved.

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
- **`Unlock`.** Cut from MVP scope during implementation scoping — a
  decided Match's `Point` freeze is unconditional (only `Undo` reverses
  it). Resuming scoring past a decided Match under non-standard rules was
  considered (two candidate state representations were discussed) but
  dropped rather than add state with no consumer yet; revisit if a real
  need shows up, informed by what slice 2 (Display) or actual usage turns
  out to require.
- Anything hardware- or network-facing.

## Deliverables

Source split by concern (`CLAUDE.md`'s code-organization convention), each
with a mirrored test module:

- A **data-shapes module**: `Side`, `SetResult`, `UndoSnapshot`,
  `MatchState`, `Command`, and the constants below. `SetResult`,
  `UndoSnapshot`, and `MatchState` each need a way to compare two values
  for equality (`CLAUDE.md`'s whole-object-assert convention needs it);
  `MatchState`'s compares scalars directly and, for `history`/`undoStack`,
  only the live prefix (`[0, historyCount)` / `[0, undoCount)`) — entries
  past the count reflect stale, no-longer-live pushes, not current state.
- A **scoring module**: applying a `Point`, Deuce threshold check.
- A **serve module**: server computation from `firstServerThisSet` +
  points played, `Set-server`'s re-anchoring math.
- An **undo module**: snapshot push/pop, restoring a `MatchState` from an
  `UndoSnapshot`.
- A **Set/Match progression module**: Set completion (history append,
  score reset, `firstServerThisSet` alternation) and Match-decided
  detection.
- A thin **`apply()` dispatcher**: switches on the command's type,
  delegates to the modules above.
- A **test module per source module above**, mirroring the split (the
  dispatcher's tests cover dispatch/no-op cases — `Start-match` while
  In-Match, unrecognized transitions — not scoring/serve/undo mechanics,
  which live in their own modules' tests).
- Build/test tooling wired up to run that suite on the host machine.

See [software-design.md](../software-design.md) and
[ADR-0006](../adr/0006-rust-for-lib-core.md) for how these map to actual
files, module names, and commands.

## Constants

- `POINTS_TO_WIN = 11` (fixed — see match-rules.md).
- `MAX_SETS = 11` — sized to the capped maximum `bestOf` (odd, ≤ 11), not
  the original "3/5/7" framing.
- `MAX_UNDO = 200` — generous headroom for a full Match's worth of points;
  the ceiling behaves as "oldest undo capability quietly stops being
  available" if ever hit, not an error.
- `NAME_LEN = 64` (bytes) — sized for UTF-8 headroom (up to 4 bytes per
  character), not just plain ASCII. Validating that a name's *byte*
  length fits within `NAME_LEN` — not character count, which would
  undercount for non-ASCII names — is a REST-boundary validation concern
  (see "Explicitly out of scope"); this slice just needs the buffer to
  exist.
- All four fit in 8 bits except `MAX_UNDO` and the undo-stack count, which
  need 16.

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
12. `Set-server {side}` sets the currently-computed server to `side`,
    pushing onto the undo stack like `Point` does, and a subsequent
    `Point` continues auto-rotation correctly from that corrected point
    (not from square one).
13. `Undo` reverses a `Set-server` correction — `server`/
    `firstServerThisSet` revert to what they were immediately before that
    correction.
14. **Once decided, `Set-server` leaves state unchanged too** — same
    freeze as `Point` (except via `Undo`); a decided Match's server
    display isn't correctable either.
15. `Start-match` from Standby succeeds: sets names/bestOf, resets all
    scores/history/undo, transitions to In-Match.
16. `Start-match` while already In-Match is a no-op (state unchanged) —
    matches the general "doesn't apply here" identity-transform rule,
    same as any other command that doesn't apply.
17. `Close` returns to Standby and clears the undo stack, regardless of
    whether the Match was decided.
18. `Point`, `Undo`, and `Set-server` are all no-ops (state unchanged)
    while in Standby — there's no Match in progress for them to act on.
    Only `Start-match` does anything from Standby.
