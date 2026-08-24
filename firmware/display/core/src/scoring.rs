//! Applying a `Point`. Set completion and the decided-freeze are separate
//! modules layered on top in later checkpoints — see
//! `docs/slices/01-backend-logic-requirements.md`.

use crate::serve::compute_server;
use crate::state::{MatchState, Side};
use crate::undo::push_undo_snapshot;

/// Increments `side`'s score by 1, leaves the other Side untouched, and
/// recomputes `server`. Pushes an undo snapshot of the pre-point state
/// first, so `Undo` can reverse it.
pub fn apply_point(state: &MatchState, side: Side) -> MatchState {
    let mut next = push_undo_snapshot(state);

    match side {
        Side::Left => next.score_left += 1,
        Side::Right => next.score_right += 1,
    }

    next.server = compute_server(next.first_server_this_set, next.score_left, next.score_right);

    next
}
