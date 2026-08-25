//! The undo stack's snapshot mechanics. See `docs/match-rules.md`'s "Undo"
//! and `docs/adr/0006-rust-for-lib-core.md`.

use crate::state::{MAX_UNDO, MatchState, UndoSnapshot};

/// Captures the fields `Undo` needs to restore into a new `UndoSnapshot`
/// and appends it to `undo_stack`, ahead of a `Point` or `Set-server`
/// mutating them. Once `MAX_UNDO` is reached, appending evicts the oldest
/// snapshot — the ceiling behaves as "oldest undo capability quietly stops
/// being available," not an error.
pub fn push_undo_snapshot(state: &MatchState) -> MatchState {
    let snapshot = UndoSnapshot {
        score_left: state.score_left,
        score_right: state.score_right,
        sets_won_left: state.sets_won_left,
        sets_won_right: state.sets_won_right,
        server: state.server,
        first_server_this_set: state.first_server_this_set,
        history_count: state.history_count,
    };

    let mut next = state.clone();
    if next.undo_count == MAX_UNDO {
        next.undo_stack.copy_within(1.., 0);
        next.undo_stack[(MAX_UNDO - 1) as usize] = snapshot;
    } else {
        next.undo_stack[next.undo_count as usize] = snapshot;
        next.undo_count += 1;
    }
    next
}

/// Pops the most recent `UndoSnapshot` and restores its fields — reversing
/// whatever `Point` or `Set-server` correction pushed it, including
/// un-deciding the Match if the popped point was what decided it. A no-op
/// (state unchanged) on an empty stack, or if there's no Match in progress
/// (Standby) — `Undo` is scoped to the current Match (see
/// `docs/match-rules.md`'s "Undo"), and `Close` already clears the stack,
/// but this guards the rule directly rather than relying on that as an
/// incidental side effect.
pub fn apply_undo(state: &MatchState) -> MatchState {
    if !state.active || state.undo_count == 0 {
        return state.clone();
    }

    let mut next = state.clone();
    next.undo_count -= 1;
    let snapshot = next.undo_stack[next.undo_count as usize];

    next.score_left = snapshot.score_left;
    next.score_right = snapshot.score_right;
    next.sets_won_left = snapshot.sets_won_left;
    next.sets_won_right = snapshot.sets_won_right;
    next.server = snapshot.server;
    next.first_server_this_set = snapshot.first_server_this_set;
    next.history_count = snapshot.history_count;

    next
}
