//! Pins `set_progression`'s three responsibilities: detecting a Set win
//! (11+, win-by-2 — covers both a plain finish and a Deuce-extended one),
//! the mechanics of completing a Set, and detecting a decided Match. See
//! test scenarios 4, 5, 6 in `docs/slices/01-backend-logic-requirements.md`.

use display_core::{MatchState, Side, check_set_winner, complete_set, is_match_decided};

mod check_set_winner_tests {
    use super::*;

    #[test]
    fn wins_at_eleven_with_two_point_lead() {
        assert_eq!(check_set_winner(11, 9), Some(Side::Left));
    }

    #[test]
    fn does_not_win_at_eleven_with_one_point_lead() {
        assert_eq!(check_set_winner(11, 10), None);
    }

    #[test]
    fn wins_beyond_eleven_in_deuce_with_two_point_lead() {
        assert_eq!(check_set_winner(13, 11), Some(Side::Left));
    }

    #[test]
    fn right_side_can_win_too() {
        assert_eq!(check_set_winner(9, 11), Some(Side::Right));
    }
}

mod complete_set_tests {
    use super::*;

    #[test]
    fn completes_a_set_won_by_left() {
        let state = MatchState {
            score_left: 11,
            score_right: 9,
            sets_won_left: 0,
            sets_won_right: 1,
            history_count: 1,
            first_server_this_set: Side::Left,
            ..MatchState::default()
        };

        let next = complete_set(&state, Side::Left);

        let mut expected = MatchState {
            score_left: 0,
            score_right: 0,
            sets_won_left: 1,
            sets_won_right: 1,
            history_count: 2,
            first_server_this_set: Side::Right, // alternates from the Set just completed
            server: Side::Right,
            ..state.clone()
        };
        expected.history[1] = display_core::SetResult {
            score_left: 11,
            score_right: 9,
        };

        assert_eq!(next, expected);
    }

    #[test]
    fn completes_a_set_won_by_right() {
        let state = MatchState {
            score_left: 9,
            score_right: 11,
            first_server_this_set: Side::Right,
            ..MatchState::default()
        };

        let next = complete_set(&state, Side::Right);

        let mut expected = MatchState {
            score_left: 0,
            score_right: 0,
            sets_won_left: 0,
            sets_won_right: 1,
            history_count: 1,
            first_server_this_set: Side::Left, // alternates from the Set just completed
            server: Side::Left,
            ..state.clone()
        };
        expected.history[0] = display_core::SetResult {
            score_left: 9,
            score_right: 11,
        };

        assert_eq!(next, expected);
    }
}

mod is_match_decided_tests {
    use super::*;

    #[test]
    fn decided_once_left_reaches_majority() {
        let state = MatchState {
            best_of: 5,
            sets_won_left: 3,
            sets_won_right: 1,
            ..MatchState::default()
        };
        assert!(is_match_decided(&state));
    }

    #[test]
    fn decided_once_right_reaches_majority() {
        let state = MatchState {
            best_of: 5,
            sets_won_left: 1,
            sets_won_right: 3,
            ..MatchState::default()
        };
        assert!(is_match_decided(&state));
    }

    #[test]
    fn not_decided_one_set_below_majority() {
        let state = MatchState {
            best_of: 5,
            sets_won_left: 2,
            sets_won_right: 2,
            ..MatchState::default()
        };
        assert!(!is_match_decided(&state));
    }
}
