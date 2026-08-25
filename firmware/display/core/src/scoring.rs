//! Applying a `Point`. See `docs/match-rules.md` and test scenarios 1, 4,
//! 5, 6, 7 in `docs/slices/01-backend-logic-requirements.md`.

use crate::serve::compute_server;
use crate::set_progression::{check_set_winner, complete_set, is_match_decided};
use crate::state::{MatchState, Side};
use crate::undo::push_undo_snapshot;

/// Increments `side`'s score by 1, leaves the other Side untouched, and
/// recomputes `server`; completes the Set if the new score calls for it.
/// Pushes an undo snapshot of the pre-point state first, so `Undo` can
/// reverse it. A no-op (state unchanged) if there's no Match in progress
/// (Standby), or if the Match is already decided — the decided check runs
/// on the *incoming* state, so the Point that decides the Match still
/// applies normally; only Points received afterward freeze.
pub fn apply_point(state: &MatchState, side: Side) -> MatchState {
    if !state.active || is_match_decided(state) {
        return state.clone();
    }

    let mut next = push_undo_snapshot(state);

    match side {
        Side::Left => next.score_left += 1,
        Side::Right => next.score_right += 1,
    }

    next.server = compute_server(next.first_server_this_set, next.score_left, next.score_right);

    if let Some(winner) = check_set_winner(next.score_left, next.score_right) {
        next = complete_set(&next, winner);
    }

    next
}
