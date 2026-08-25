//! Server computation from `first_server_this_set` + points played this
//! Set, and `Set-server`'s re-anchoring correction — see
//! `docs/match-rules.md`'s "Serve rotation".

use crate::set_progression::is_match_decided;
use crate::state::{MatchState, Side};
use crate::undo::push_undo_snapshot;

/// Server alternates every 2 points, except once both sides have reached
/// 10 (Deuce), when it alternates every single point instead.
fn is_deuce(score_left: u8, score_right: u8) -> bool {
    score_left >= 10 && score_right >= 10
}

/// 0 if the anchor Side is currently due to serve, 1 if it's the other
/// Side's turn — the one bit `compute_server` and `solve_first_server_for`
/// both turn on.
fn rotation_parity(score_left: u8, score_right: u8) -> u8 {
    let total = score_left as u16 + score_right as u16;

    // Deuce (both sides at 10+) always starts at exactly 20 points, and
    // 20 / 2 = 10 is even, so whoever the normal every-2 rotation would
    // have serving at that point is also who starts the every-1 rotation
    // — no special-casing needed for continuity at the boundary.
    if is_deuce(score_left, score_right) {
        ((total - 20) % 2) as u8
    } else {
        ((total / 2) % 2) as u8
    }
}

/// The Side currently due to serve, derived from `first_server_this_set`
/// and the current score — never stored/incremented directly (see
/// `docs/architecture.md`'s "Serve computation").
pub fn compute_server(first_server_this_set: Side, score_left: u8, score_right: u8) -> Side {
    if rotation_parity(score_left, score_right) == 0 {
        first_server_this_set
    } else {
        first_server_this_set.other()
    }
}

/// The `first_server_this_set` value that makes
/// `compute_server(_, score_left, score_right)` come out to `requested_side`
/// — used by `apply_set_server` to re-anchor rotation around a correction
/// instead of just overwriting `server` and going stale on the next point.
///
/// This is the exact same formula as `compute_server` — flipping on odd
/// parity is self-inverse, so "solve for the anchor that produces this
/// server" and "compute the server from this anchor" are the same
/// operation. Kept as a separately-named function for clarity at the call
/// site, not because the logic actually differs.
pub fn solve_first_server_for(requested_side: Side, score_left: u8, score_right: u8) -> Side {
    compute_server(requested_side, score_left, score_right)
}

/// Corrects `server` to `side`, re-anchoring `first_server_this_set` so
/// auto-rotation continues correctly from here rather than resetting.
/// Pushes an undo snapshot first, same as `Point`. A no-op (state
/// unchanged) if there's no Match in progress (Standby), or if the Match
/// is already decided — same freeze as `Point`.
pub fn apply_set_server(state: &MatchState, side: Side) -> MatchState {
    if !state.active || is_match_decided(state) {
        return state.clone();
    }

    let mut next = push_undo_snapshot(state);
    next.first_server_this_set = solve_first_server_for(side, next.score_left, next.score_right);
    next.server = side;
    next
}
