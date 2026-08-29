//! `state.rs`'s `PartialEq` for `MatchState` is what the
//! whole-object-assert testing convention (`CLAUDE.md`) depends on for
//! every other test module — pin its behavior down directly, including the
//! "only the live prefix matters" rule for the `history`/`undo_stack`
//! arrays (see `docs/slices/01-backend-logic-requirements.md`).

use display_core::{MatchState, SetResult};

mod match_state_equality {
    use super::*;

    #[test]
    fn differing_score_is_not_equal() {
        let a = MatchState::default();
        let b = MatchState {
            score_left: 3,
            ..MatchState::default()
        };

        assert_ne!(a, b);
    }

    #[test]
    fn differing_live_history_entry_is_not_equal() {
        let mut a = MatchState {
            history_count: 1,
            ..MatchState::default()
        };
        a.history[0] = SetResult {
            score_left: 11,
            score_right: 9,
        };

        let mut b = MatchState {
            history_count: 1,
            ..MatchState::default()
        };
        b.history[0] = SetResult {
            score_left: 11,
            score_right: 7,
        };

        assert_ne!(a, b);
    }

    #[test]
    fn differs_only_past_history_count_are_still_equal() {
        let mut a = MatchState {
            history_count: 0,
            ..MatchState::default()
        };
        a.history[3] = SetResult {
            score_left: 11,
            score_right: 9,
        }; // stale, past the live prefix

        let mut b = MatchState {
            history_count: 0,
            ..MatchState::default()
        };
        b.history[3] = SetResult {
            score_left: 5,
            score_right: 2,
        }; // different stale contents, same prefix

        assert_eq!(a, b);
    }

    #[test]
    fn differs_only_past_undo_count_are_still_equal() {
        let mut a = MatchState {
            undo_count: 0,
            ..MatchState::default()
        };
        a.undo_stack[10].score_left = 7;

        let mut b = MatchState {
            undo_count: 0,
            ..MatchState::default()
        };
        b.undo_stack[10].score_left = 2;

        assert_eq!(a, b);
    }
}
