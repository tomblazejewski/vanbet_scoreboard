//! Pins `undo::push_undo_snapshot`'s two responsibilities: capture the
//! pre-push state into a new `UndoSnapshot`, and evict the oldest entry
//! once `MAX_UNDO` is reached (`docs/slices/01-backend-logic-requirements.md`'s
//! `MAX_UNDO` constant note).

use display_core::{MAX_UNDO, MatchState, Side, UndoSnapshot, push_undo_snapshot};

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
