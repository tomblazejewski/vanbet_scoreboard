# Match rules (as implemented)

Fixed rules, not configurable — see the "explicitly out of scope" note in
[architecture.md](architecture.md). Terms below are defined in
[`CONTEXT.md`](../CONTEXT.md).

## Scoring a Set

- First to 11 points wins the Set, **unless** the score reaches 10-10
  (Deuce) — then play continues until one Side leads by 2.
- A Set win: append `{scoreLeft, scoreRight}` to `history`, increment the
  winner's `setsWonLeft`/`setsWonRight`, reset both current scores to 0.

## Winning the Match

- First to win a majority of `bestOf` Sets (e.g. `bestOf: 5` → first to 3)
  wins the Match. Once reached, the Match is *decided* — the Display shows
  the final result, but stays In-Match (not Standby) until an explicit
  `Close` (see [ADR-0002](adr/0002-persist-and-resume-explicit-close.md)).
- Once decided, `Point` stops changing anything — the score is frozen at
  the final result, unconditionally. `Undo` is the one exception: it still
  works, since reversing a mistaken match-ending point is exactly the
  scenario it exists for (see "Undo" below). There is no other way to
  resume scoring past a decided Match — an `Unlock`-style override was
  considered and cut from MVP scope; see
  [slices/01-backend-logic-requirements.md](slices/01-backend-logic-requirements.md).

## Serve rotation

- Serve alternates every 2 points for the whole Set.
- **Exception:** once the score reaches 10-10 (Deuce), serve alternates
  every single point instead.
- The Side that served first in a Set is *not* the Side that serves first in
  the next Set — first server alternates Set to Set.
- `server` is always derived from `firstServerThisSet` + points played this
  Set, never stored/incremented directly — see architecture.md's "Serve
  computation" section for exactly how `SET_SERVER` re-anchors this so
  auto-rotation keeps working correctly after a manual correction.

## Undo

A real stack, not single-level: repeated `UNDO` walks back through points
one at a time, including reversing a Set that just completed (removing it
from `history`, restoring the pre-completion score, decrementing
`setsWonLeft`/`setsWonRight`) if the winning point turns out to have been a
mistake. Scoped to the current Match — the stack is cleared on `Close`.

`Undo` works even once the Match is decided (unlike `Point`, which freezes)
— it's the mechanism for fixing a match-ending point that shouldn't have
counted, so it can't itself be blocked by the state it needs to undo.

`SET_SERVER` corrections are deliberately **not** on the undo stack — undo
is for point mistakes; a wrong server assignment is fixed by calling
`SET_SERVER` again with the right Side, not by undoing.
