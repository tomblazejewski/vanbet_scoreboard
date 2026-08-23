# Match state persists across reboots and resumes silently; only an explicit close ends it

The Display writes Match state (Set format is fixed, but scores, Sets played,
Server, and undo history are not) to flash after every point. On boot, if
saved state exists, the Display resumes it immediately and silently — no
"is this stale?" check, no timer, no confirmation step. A Match only ends
when something explicitly calls the close-Match endpoint, whether that's
right after a natural win (majority of Sets reached — the Display shows the
final result and just waits) or as a manual abandon mid-Match.

**Considered and rejected:** timing out back to an idle screen automatically
after a win, or after a period of inactivity — would need to invent a
threshold with no principled value, and risks clearing a Match that's still
being looked at (e.g. players stepped away to grab a drink after the last
point).

**Consequence:** there's no automatic cleanup — if nobody ever calls close,
the Display will resume showing the same finished Match indefinitely across
any number of reboots. That's intentional: explicit is simpler to reason
about than any staleness heuristic we could invent.
