//! Pins `scoring::apply_point`'s basic behavior: increment the scoring
//! Side, leave the other untouched, recompute `server`, and push an undo
//! snapshot of the pre-point state. Set completion and the decided-freeze
//! aren't wired in yet (later checkpoints) — see test scenario 1 in
//! `docs/slices/01-backend-logic-requirements.md`.

use display_core::{MatchState, Side, UndoSnapshot, apply_point};

mod apply_point_tests {
    use super::*;

    #[test]
    fn a_point_for_left_increments_left_and_leaves_right_untouched() {
        let state = MatchState::default(); // 0-0, first_server_this_set = Left

        let next = apply_point(&state, Side::Left);

        let mut expected = MatchState {
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
        let state = MatchState::default();

        let next = apply_point(&state, Side::Right);

        let mut expected = MatchState {
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
