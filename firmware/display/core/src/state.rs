//! `MatchState` and its supporting data shapes for the Display's
//! state-manipulation core. See `docs/architecture.md`'s "Data model" and
//! `docs/slices/01-backend-logic-requirements.md`.

/// Fixed rules — see docs/match-rules.md. Not configurable.
pub const POINTS_TO_WIN: u8 = 11;

/// Sized to the capped maximum bestOf (odd, <= 11) — see
/// docs/slices/01-backend-logic-requirements.md.
pub const MAX_SETS: u8 = 11;

/// Generous headroom for a full Match's worth of points. Once full,
/// pushing a new snapshot evicts the oldest — the ceiling behaves as
/// "oldest undo capability quietly stops being available," not an error.
/// See the undo module.
pub const MAX_UNDO: u16 = 200;

/// Buffer size for a Player name. Length itself is a REST-boundary
/// validation concern (slice 3) — this just needs a buffer big enough for
/// whatever the REST layer already validated.
pub const NAME_LEN: usize = 16;

/// Left or Right — fixed for the whole Match by which Controller button is
/// pressed, not by player identity. See CONTEXT.md.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Side {
    #[default]
    Left,
    Right,
}

impl Side {
    /// The other Side — Left <-> Right.
    pub fn other(self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

/// A completed Set's final score, appended to `MatchState::history`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SetResult {
    pub score_left: u8,
    pub score_right: u8,
}

/// Pushed before every `Point` or `Set-server` is applied; popped +
/// restored on Undo. Both mutate server-rotation state (and `Point` may
/// also mutate score/history/setsWon on a Set completion), and `Undo`
/// covers reversing either kind of mistake.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UndoSnapshot {
    pub score_left: u8,
    pub score_right: u8,
    pub sets_won_left: u8,
    pub sets_won_right: u8,
    pub server: Side,
    pub first_server_this_set: Side,
    pub history_count: u8,
}

/// The Display's sole authoritative state. `active == false` is Standby;
/// `active == true` is In-Match (whether or not the Match is decided — see
/// `docs/architecture.md`'s "Match lifecycle").
///
/// Invariant: `history_count <= MAX_SETS` and `undo_count <= MAX_UNDO`.
/// `undo_count` is maintained within bound structurally (`undo::push_undo_snapshot`'s
/// ring-buffer eviction can't push it past `MAX_UNDO`); `history_count` relies
/// on `best_of <= MAX_SETS` holding, which is a REST-boundary validation
/// concern (slice 3) — this crate assumes it, doesn't enforce it, and
/// doesn't defend against a `MatchState` that violates it (e.g. corrupted
/// persisted state, or one hand-built without going through `apply()`).
#[derive(Clone, Debug)]
pub struct MatchState {
    pub active: bool,
    pub name_left: [u8; NAME_LEN],
    pub name_right: [u8; NAME_LEN],
    pub best_of: u8,
    pub sets_won_left: u8,
    pub sets_won_right: u8,
    pub score_left: u8, // current Set in progress
    pub score_right: u8,
    pub server: Side,
    pub first_server_this_set: Side, // anchor for computing server
    pub history: [SetResult; MAX_SETS as usize],
    pub history_count: u8,
    pub undo_stack: [UndoSnapshot; MAX_UNDO as usize],
    pub undo_count: u16,
}

impl Default for MatchState {
    fn default() -> Self {
        MatchState {
            active: false,
            name_left: [0; NAME_LEN],
            name_right: [0; NAME_LEN],
            best_of: 0,
            sets_won_left: 0,
            sets_won_right: 0,
            score_left: 0,
            score_right: 0,
            server: Side::default(),
            first_server_this_set: Side::default(),
            history: [SetResult::default(); MAX_SETS as usize],
            history_count: 0,
            undo_stack: [UndoSnapshot::default(); MAX_UNDO as usize],
            undo_count: 0,
        }
    }
}

impl PartialEq for MatchState {
    /// Compares scalar fields directly; for `history`/`undo_stack`, only
    /// the live prefix (`[0, history_count)` / `[0, undo_count)`) — entries
    /// past the count reflect stale, no-longer-live pushes, not current
    /// state.
    fn eq(&self, other: &Self) -> bool {
        if self.active != other.active
            || self.name_left != other.name_left
            || self.name_right != other.name_right
            || self.best_of != other.best_of
            || self.sets_won_left != other.sets_won_left
            || self.sets_won_right != other.sets_won_right
            || self.score_left != other.score_left
            || self.score_right != other.score_right
            || self.server != other.server
            || self.first_server_this_set != other.first_server_this_set
            || self.history_count != other.history_count
            || self.undo_count != other.undo_count
        {
            return false;
        }

        let history_count = self.history_count as usize;
        if self.history[..history_count] != other.history[..history_count] {
            return false;
        }

        let undo_count = self.undo_count as usize;
        self.undo_stack[..undo_count] == other.undo_stack[..undo_count]
    }
}
