//! Pins `serve::compute_server`'s rotation math (every 2 points in normal
//! play, every 1 point once both sides reach 10 — Deuce) and
//! `serve::apply_set_server`'s re-anchoring math. See `docs/match-rules.md`
//! and test scenarios 2, 3, 12, 14 in
//! `docs/slices/01-backend-logic-requirements.md`.

use display_core::{MatchState, Side, UndoSnapshot, apply_point, apply_set_server, compute_server};

mod compute_server_tests {
    use super::*;

    #[test]
    fn at_match_start_first_server_this_set_serves() {
        assert_eq!(compute_server(Side::Left, 0, 0), Side::Left);
    }

    #[test]
    fn first_server_still_serves_the_second_point_of_their_turn() {
        assert_eq!(compute_server(Side::Left, 1, 0), Side::Left);
    }

    #[test]
    fn server_switches_after_two_points_in_normal_play() {
        assert_eq!(compute_server(Side::Left, 1, 1), Side::Right);
    }

    #[test]
    fn server_switches_back_after_four_points_in_normal_play() {
        assert_eq!(compute_server(Side::Left, 2, 2), Side::Left);
    }

    #[test]
    fn server_at_deuce_onset_continues_from_normal_rotation() {
        // 10-10 is reached after 20 points — the same point count that
        // would put the normal every-2 rotation back on first_server_this_set,
        // so there's no discontinuity right at the Deuce boundary.
        assert_eq!(compute_server(Side::Left, 10, 10), Side::Left);
    }

    #[test]
    fn server_switches_every_single_point_once_in_deuce() {
        // Only one point past the 10-10 boundary — under the normal every-2
        // rule this wouldn't have switched yet, so this pins the Deuce
        // exception specifically.
        assert_eq!(compute_server(Side::Left, 11, 10), Side::Right);
    }
}

mod apply_set_server_tests {
    use super::*;

    #[test]
    fn sets_server_to_the_requested_side_and_reanchors_first_server_this_set() {
        let state = MatchState {
            active: true,
            score_left: 3,
            score_right: 2,
            server: Side::Left,
            first_server_this_set: Side::Left,
            ..MatchState::default()
        };

        let next = apply_set_server(&state, Side::Right);

        let mut expected = MatchState {
            server: Side::Right,
            first_server_this_set: Side::Right,
            undo_count: 1,
            ..state.clone()
        };
        expected.undo_stack[0] = UndoSnapshot {
            score_left: 3,
            score_right: 2,
            server: Side::Left,
            first_server_this_set: Side::Left,
            ..UndoSnapshot::default()
        };

        assert_eq!(next, expected);
    }

    #[test]
    fn reanchors_correctly_when_current_parity_is_odd() {
        // At 2-0, compute_server(Left, 2, 0) == Right (parity is odd here) —
        // requesting Left back means first_server_this_set must flip to
        // Right, not just be set to Left, to make the math come out right.
        let state = MatchState {
            active: true,
            score_left: 2,
            score_right: 0,
            server: Side::Right,
            first_server_this_set: Side::Left,
            ..MatchState::default()
        };

        let next = apply_set_server(&state, Side::Left);

        let mut expected = MatchState {
            server: Side::Left,
            first_server_this_set: Side::Right,
            undo_count: 1,
            ..state.clone()
        };
        expected.undo_stack[0] = UndoSnapshot {
            score_left: 2,
            score_right: 0,
            server: Side::Right,
            first_server_this_set: Side::Left,
            ..UndoSnapshot::default()
        };

        assert_eq!(next, expected);
        assert_eq!(compute_server(next.first_server_this_set, 2, 0), Side::Left);
    }

    #[test]
    fn a_subsequent_point_continues_rotation_from_the_corrected_anchor() {
        let state = MatchState {
            active: true,
            score_left: 3,
            score_right: 2,
            server: Side::Left,
            first_server_this_set: Side::Left,
            ..MatchState::default()
        };

        let corrected = apply_set_server(&state, Side::Right);
        let next = apply_point(&corrected, Side::Left);

        // Without the re-anchor (i.e. if first_server_this_set had stayed
        // Left), the next point's server would come out Right instead —
        // this is the property scenario 12 is actually about.
        assert_eq!(next.server, Side::Left);
    }

    #[test]
    fn once_decided_set_server_leaves_state_unchanged() {
        let state = MatchState {
            active: true,
            best_of: 3,
            sets_won_left: 2, // majority of 3 already reached
            server: Side::Left,
            first_server_this_set: Side::Left,
            ..MatchState::default()
        };

        let next = apply_set_server(&state, Side::Right);

        assert_eq!(next, state, "frozen: no undo push, no server change");
    }

    #[test]
    fn is_a_no_op_while_no_match_is_active() {
        let state = MatchState {
            server: Side::Left,
            ..MatchState::default()
        }; // active: false

        assert_eq!(apply_set_server(&state, Side::Right), state);
    }
}
