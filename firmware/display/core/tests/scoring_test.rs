//! Pins `scoring::apply_point`'s behavior: increment the scoring Side,
//! leave the other untouched, recompute `server`, push an undo snapshot of
//! the pre-point state, complete a Set when the score calls for it, and
//! freeze once the Match is already decided. See test scenarios 1, 4, 5,
//! 6, 7 in `docs/slices/01-backend-logic-requirements.md`.

use display_core::{MatchState, SetResult, Side, UndoSnapshot, apply_point};

mod apply_point_tests {
    use super::*;

    #[test]
    fn a_point_for_left_increments_left_and_leaves_right_untouched() {
        let state = MatchState { active: true, ..MatchState::default() }; // 0-0, first_server_this_set = Left

        let next = apply_point(&state, Side::Left);

        let mut expected = MatchState {
            active: true,
            score_left: 1,
            score_right: 0,
            server: Side::Left, // 1 point played, still first_server_this_set's turn
            undo_count: 1,
            ..MatchState::default()
        };
        expected.undo_stack[0] = UndoSnapshot::default(); // pre-point state was all-default

        assert_eq!(next, expected);
    }

    #[test]
    fn a_point_for_right_increments_right_and_leaves_left_untouched() {
        let state = MatchState { active: true, ..MatchState::default() };

        let next = apply_point(&state, Side::Right);

        let mut expected = MatchState {
            active: true,
            score_left: 0,
            score_right: 1,
            server: Side::Left, // 1 point played, still first_server_this_set's turn
            undo_count: 1,
            ..MatchState::default()
        };
        expected.undo_stack[0] = UndoSnapshot::default();

        assert_eq!(next, expected);
    }
}

mod inactive_no_op_tests {
    use super::*;

    #[test]
    fn a_point_is_a_no_op_while_no_match_is_active() {
        let state = MatchState { score_left: 3, ..MatchState::default() }; // active: false

        assert_eq!(apply_point(&state, Side::Left), state);
    }
}

mod set_completion_tests {
    use super::*;

    #[test]
    fn a_point_reaching_eleven_with_two_point_lead_completes_the_set() {
        let state = MatchState {
            active: true,
            score_left: 10,
            score_right: 9,
            server: Side::Right,
            first_server_this_set: Side::Left,
            best_of: 5,
            ..MatchState::default()
        };

        let next = apply_point(&state, Side::Left);

        let mut expected = MatchState {
            score_left: 0,
            score_right: 0,
            sets_won_left: 1,
            sets_won_right: 0,
            history_count: 1,
            first_server_this_set: Side::Right, // alternates from the Set just completed
            server: Side::Right,
            undo_count: 1,
            ..state.clone()
        };
        expected.history[0] = SetResult { score_left: 11, score_right: 9 };
        expected.undo_stack[0] = UndoSnapshot {
            score_left: 10,
            score_right: 9,
            server: Side::Right,
            first_server_this_set: Side::Left,
            ..UndoSnapshot::default()
        };

        assert_eq!(next, expected);
    }

    #[test]
    fn a_point_reaching_eleven_with_one_point_lead_does_not_complete_the_set() {
        let state = MatchState {
            active: true,
            score_left: 10,
            score_right: 10, // Deuce
            server: Side::Left,
            first_server_this_set: Side::Left,
            best_of: 5,
            ..MatchState::default()
        };

        let next = apply_point(&state, Side::Left);

        let mut expected = MatchState {
            score_left: 11,
            score_right: 10,
            server: Side::Right, // every-1-point Deuce rotation
            undo_count: 1,
            ..state.clone()
        };
        expected.undo_stack[0] = UndoSnapshot {
            score_left: 10,
            score_right: 10,
            server: Side::Left,
            first_server_this_set: Side::Left,
            ..UndoSnapshot::default()
        };

        assert_eq!(next, expected);
    }
}

mod decided_freeze_tests {
    use super::*;

    #[test]
    fn once_decided_a_point_leaves_state_unchanged() {
        let state = MatchState {
            active: true,
            best_of: 3,
            sets_won_left: 2, // majority of 3 already reached
            sets_won_right: 0,
            score_left: 3,
            score_right: 2,
            history_count: 2,
            ..MatchState::default()
        };

        let next = apply_point(&state, Side::Left);

        assert_eq!(next, state, "frozen: no undo push, no score change");
    }

    #[test]
    fn the_point_that_wins_the_deciding_set_still_applies_normally() {
        // Not yet decided going in (1 of 2 needed Sets) — the freeze only
        // applies to Points received *after* the Match is already decided,
        // not to the Point that decides it.
        let state = MatchState {
            active: true,
            best_of: 3,
            sets_won_left: 1,
            sets_won_right: 0,
            score_left: 10,
            score_right: 9,
            server: Side::Right,
            first_server_this_set: Side::Left,
            history_count: 1,
            ..MatchState::default()
        };

        let next = apply_point(&state, Side::Left);

        let mut expected = MatchState {
            score_left: 0,
            score_right: 0,
            sets_won_left: 2, // now decided, but this point still counted
            sets_won_right: 0,
            history_count: 2,
            first_server_this_set: Side::Right,
            server: Side::Right,
            undo_count: 1,
            ..state.clone()
        };
        expected.history[1] = SetResult { score_left: 11, score_right: 9 };
        expected.undo_stack[0] = UndoSnapshot {
            score_left: 10,
            score_right: 9,
            sets_won_left: 1,
            server: Side::Right,
            first_server_this_set: Side::Left,
            history_count: 1,
            ..UndoSnapshot::default()
        };

        assert_eq!(next, expected);
    }
}
