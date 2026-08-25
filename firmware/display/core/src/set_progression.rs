//! Set completion and Match-decided detection. See
//! `docs/match-rules.md`'s "Scoring a Set" and "Winning the Match".

use crate::serve::compute_server;
use crate::state::{MatchState, POINTS_TO_WIN, SetResult, Side};

/// A Set is won at `POINTS_TO_WIN`+ with a 2+ point lead — the same check
/// covers both a plain finish (11-9) and a Deuce-extended one (13-11),
/// since win-by-2 already subsumes the Deuce case.
pub fn check_set_winner(score_left: u8, score_right: u8) -> Option<Side> {
    let (leader, lead) = if score_left >= score_right {
        (Side::Left, score_left.abs_diff(score_right))
    } else {
        (Side::Right, score_right.abs_diff(score_left))
    };

    (score_left.max(score_right) >= POINTS_TO_WIN && lead >= 2).then_some(leader)
}

/// Appends the just-finished Set to `history`, credits `winner`'s Set-win
/// count, resets the current score to 0-0, and alternates
/// `first_server_this_set` (and therefore `server`) for the next Set.
pub fn complete_set(state: &MatchState, winner: Side) -> MatchState {
    let mut next = state.clone();

    next.history[next.history_count as usize] =
        SetResult { score_left: state.score_left, score_right: state.score_right };
    next.history_count += 1;

    match winner {
        Side::Left => next.sets_won_left += 1,
        Side::Right => next.sets_won_right += 1,
    }

    next.score_left = 0;
    next.score_right = 0;
    next.first_server_this_set = next.first_server_this_set.other();
    next.server = compute_server(next.first_server_this_set, 0, 0);

    next
}

/// The Match is decided once a Side has won a majority of `best_of` Sets.
/// A purely computed fact, never stored — see ADR-0006's note (via the
/// Unlock-removal decision) on why `Point`'s freeze doesn't need its own
/// flag.
pub fn is_match_decided(state: &MatchState) -> bool {
    let majority = state.best_of / 2 + 1;
    state.sets_won_left >= majority || state.sets_won_right >= majority
}
