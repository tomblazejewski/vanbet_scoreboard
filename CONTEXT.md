# Table Tennis Scoreboard

A wireless scoreboard for table tennis matches played in the office: an ESP32-driven
LED display shows live score, and control happens entirely off-device (phone or a
dedicated remote).

## Language

**Set**:
The unit of play worth winning outright — first to 11 points, win by 2 (see Deuce).
A Match consists of several Sets.
_Avoid_: Game (ITTF's official term, but not how this project talks)

**Match**:
The full contest between two Players — best of 3, 5, or 7 Sets, decided once one
side has won a majority of Sets. Exists only while the Display is In-Match.

**Standby**:
The Display's state when there is no Match in progress — before the first Match of
the day, or after one has ended. One of two values of the Display's state (see
In-Match).
_Avoid_: NO_MATCH, idle, active == false

**In-Match**:
The Display's state while a Match is in progress — whether or not that Match is
decided (see Match). The other of the Display's two states (see Standby).
_Avoid_: MATCH_ACTIVE, active == true

**Deuce**:
The state within a Set when the score reaches 10-10. Changes serve rotation from
2 serves per turn to 1 serve per turn; the Set continues until one side leads by 2.

**Server**:
The Player currently serving. Rotates every 2 points in normal play, every 1 point
during Deuce.

**Controller**:
The single shared physical remote, sitting at the table, with one button per side
(not per player) plus undo. Controls both Players' scoring — there is no concept of
a personal, per-player remote.
_Avoid_: Remote (use Controller for the noun; "remote" only as a loose adjective)

**Display**:
The ESP32 + LED panel unit mounted in the office. Owns the sole authoritative Match
state (scores, Sets, Server) and derives Server rotation automatically from points
added and new Sets started. The Controller and phone only ever send it discrete
commands — they hold no state of their own.

**Side**:
Left or Right — fixed for the whole Match by which Controller button is pressed, not
by player identity. A Player's name is assigned to a Side when the Match starts;
Side itself never moves, even if players would physically swap chairs mid-match.
Manually correctable (a "swap" action) for the rare case the Controller was set
down backwards.
_Avoid_: Player A/B, Player 1/2 (Side is the canonical index; a Player's name is
just a label attached to a Side for a given Match)
