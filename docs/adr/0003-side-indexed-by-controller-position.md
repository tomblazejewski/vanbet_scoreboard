# Scoring is indexed by Side (Controller button position), not by Player identity

The Controller has exactly one button per physical position — Left and Right
— not one per player. A Player's name is attached to a Side when a Match
starts and stays fixed there for the whole Match; there's no per-player
remote and no way for the Controller to know or care who's currently sitting
where. Left/Right is a manually-correctable convention (a `SET_SERVER`-style
explicit assignment covers the serve side of this; a mis-assigned Side at
Match start means re-starting the Match with names swapped, not a live
relabel).

**Considered and rejected:** a per-player remote (or a shared remote that's
somehow player-aware) so scoring could be indexed by Player identity
directly — descoped early: it would need either two Controllers (more BOM,
more to charge/carry) or some way for a single shared Controller to know
which physical button belongs to which player, which only pushes the
Left/Right convention down a level instead of removing it.

**Consequence:** all protocol and data-model fields are Side-indexed
(`left`/`right`), never player-indexed — `CONTEXT.md`'s **Side** entry is the
canonical vocabulary here, and `Player A/B`-style naming should be treated as
a glossary violation if it shows up in code or docs.
