//! Server computation from `first_server_this_set` + points played this
//! Set — see `docs/match-rules.md`'s "Serve rotation".

use crate::state::Side;

/// Server alternates every 2 points, except once both sides have reached
/// 10 (Deuce), when it alternates every single point instead.
fn is_deuce(score_left: u8, score_right: u8) -> bool {
    score_left >= 10 && score_right >= 10
}

/// The Side currently due to serve, derived from `first_server_this_set`
/// and the current score — never stored/incremented directly (see
/// `docs/architecture.md`'s "Serve computation").
pub fn compute_server(first_server_this_set: Side, score_left: u8, score_right: u8) -> Side {
    let total = score_left as u16 + score_right as u16;

    // Deuce (both sides at 10+) always starts at exactly 20 points, and
    // 20 / 2 = 10 is even, so whoever the normal every-2 rotation would
    // have serving at that point is also who starts the every-1 rotation
    // — no special-casing needed for continuity at the boundary.
    let parity = if is_deuce(score_left, score_right) {
        (total - 20) % 2
    } else {
        (total / 2) % 2
    };

    if parity == 0 {
        first_server_this_set
    } else {
        first_server_this_set.other()
    }
}
