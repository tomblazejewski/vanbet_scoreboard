//! Pins `undo::push_undo_snapshot` and `undo::apply_undo`. Push: capture
//! the pre-push state, evict the oldest entry once `MAX_UNDO` is reached
//! (`docs/slices/01-backend-logic-requirements.md`'s `MAX_UNDO` constant
//! note). Undo: pop + restore, a no-op on an empty stack, reversing
//! everything a `Point` (including a Set completion, including un-deciding
//! the Match) or a `Set-server` correction did — see test scenarios 8, 9,
//! 10, 11.

use display_core::{
    MAX_UNDO, MatchState, Side, UndoSnapshot, apply_point, apply_set_server, apply_undo,
    push_undo_snapshot,
};

mod push_undo_snapshot_tests {
    use super::*;

    #[test]
    fn appends_a_snapshot_of_the_pre_push_state_without_mutating_it() {
        let state = MatchState {
            score_left: 5,
            score_right: 3,
            sets_won_left: 1,
            sets_won_right: 0,
            server: Side::Right,
            first_server_this_set: Side::Left,
            history_count: 1,
            ..MatchState::default()
        };

        let next = push_undo_snapshot(&state);

        let expected = MatchState {
            undo_count: 1,
            undo_stack: {
                let mut stack = state.undo_stack;
                stack[0] = UndoSnapshot {
                    score_left: 5,
                    score_right: 3,
                    sets_won_left: 1,
                    sets_won_right: 0,
                    server: Side::Right,
                    first_server_this_set: Side::Left,
                    history_count: 1,
                };
                stack
            },
            ..state.clone()
        };

        assert_eq!(next, expected);
    }

    #[test]
    fn evicts_the_oldest_snapshot_once_full() {
        let mut state = MatchState { score_left: 9, ..MatchState::default() };
        state.undo_count = MAX_UNDO;
        for i in 0..MAX_UNDO as usize {
            state.undo_stack[i].score_left = i as u8;
        }

        let next = push_undo_snapshot(&state);

        assert_eq!(next.undo_count, MAX_UNDO, "count stays capped at MAX_UNDO");
        assert_eq!(
            next.undo_stack[0].score_left, 1,
            "oldest entry (previously index 0) was evicted; index 1 shifted down to index 0"
        );
        let newest = next.undo_stack[(MAX_UNDO - 1) as usize];
        assert_eq!(
            newest,
            UndoSnapshot { score_left: 9, ..UndoSnapshot::default() },
            "newest entry is the just-pushed snapshot of the pre-push state"
        );
    }
}

mod apply_undo_tests {
    use super::*;

    #[test]
    fn no_op_on_empty_stack() {
        let state = MatchState { score_left: 4, ..MatchState::default() };

        assert_eq!(apply_undo(&state), state);
    }

    #[test]
    fn pops_the_most_recent_snapshot_and_restores_its_fields() {
        let mut state = MatchState {
            score_left: 5,
            score_right: 3,
            sets_won_left: 1,
            sets_won_right: 0,
            server: Side::Left,
            first_server_this_set: Side::Right,
            history_count: 1,
            undo_count: 1,
            ..MatchState::default()
        };
        state.undo_stack[0] = UndoSnapshot {
            score_left: 2,
            score_right: 1,
            sets_won_left: 0,
            sets_won_right: 0,
            server: Side::Right,
            first_server_this_set: Side::Left,
            history_count: 0,
        };

        let next = apply_undo(&state);

        let expected = MatchState {
            score_left: 2,
            score_right: 1,
            sets_won_left: 0,
            sets_won_right: 0,
            server: Side::Right,
            first_server_this_set: Side::Left,
            history_count: 0,
            undo_count: 0,
            ..state.clone()
        };

        assert_eq!(next, expected);
    }

    #[test]
    fn undo_reverses_a_plain_point_round_trip() {
        let before = MatchState {
            score_left: 3,
            score_right: 2,
            server: Side::Left,
            first_server_this_set: Side::Left,
            best_of: 5,
            ..MatchState::default()
        };

        let after_point = apply_point(&before, Side::Right);
        let after_undo = apply_undo(&after_point);

        assert_eq!(after_undo, before);
    }

    #[test]
    fn undo_walks_back_across_a_set_boundary_reopening_the_completed_set() {
        // 10-9, Left about to win the Set.
        let before_set_win = MatchState {
            score_left: 10,
            score_right: 9,
            server: Side::Right,
            first_server_this_set: Side::Left,
            best_of: 5,
            ..MatchState::default()
        };

        let after_set_win = apply_point(&before_set_win, Side::Left); // completes the Set
        let after_next_point = apply_point(&after_set_win, Side::Left); // first point of the new Set

        let undo_once = apply_undo(&after_next_point);
        assert_eq!(undo_once, after_set_win, "first undo removes the new-Set point");

        let undo_twice = apply_undo(&undo_once);
        assert_eq!(undo_twice, before_set_win, "second undo reopens the just-completed Set");
    }

    #[test]
    fn undo_un_decides_the_match_by_restoring_the_pre_point_state() {
        // 1 of 2 needed Sets already won, 10-9 — this Point will decide the Match.
        let before_deciding_point = MatchState {
            score_left: 10,
            score_right: 9,
            sets_won_left: 1,
            server: Side::Right,
            first_server_this_set: Side::Left,
            best_of: 3,
            ..MatchState::default()
        };

        let after_deciding_point = apply_point(&before_deciding_point, Side::Left);
        let after_undo = apply_undo(&after_deciding_point);

        assert_eq!(after_undo, before_deciding_point);
    }

    #[test]
    fn undo_reverses_a_set_server_correction() {
        let before = MatchState {
            score_left: 3,
            score_right: 2,
            server: Side::Left,
            first_server_this_set: Side::Left,
            ..MatchState::default()
        };

        let after_correction = apply_set_server(&before, Side::Right);
        let after_undo = apply_undo(&after_correction);

        assert_eq!(after_undo, before);
    }
}
