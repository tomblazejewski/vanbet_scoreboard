//! Pins `lifecycle::apply_start_match` and `lifecycle::apply_close` — the
//! Standby <-> In-Match transition. See test scenarios 15, 16, 17 in
//! `docs/slices/01-backend-logic-requirements.md`.

use display_core::{MatchState, NAME_LEN, apply_close, apply_start_match};

fn name_bytes(name: &str) -> [u8; NAME_LEN] {
    let mut buf = [0u8; NAME_LEN];
    buf[..name.len()].copy_from_slice(name.as_bytes());
    buf
}

mod apply_start_match_tests {
    use super::*;

    #[test]
    fn starts_a_match_from_standby() {
        let state = MatchState::default(); // active: false

        let next = apply_start_match(&state, "Alice", "Bob", 5);

        let expected = MatchState {
            active: true,
            name_left: name_bytes("Alice"),
            name_right: name_bytes("Bob"),
            best_of: 5,
            ..MatchState::default()
        };

        assert_eq!(next, expected);
    }

    #[test]
    fn is_a_no_op_while_already_in_match() {
        let state = MatchState {
            active: true,
            score_left: 3,
            name_left: name_bytes("Old"),
            name_right: name_bytes("Match"),
            best_of: 3,
            ..MatchState::default()
        };

        let next = apply_start_match(&state, "New", "Names", 7);

        assert_eq!(next, state);
    }
}

mod apply_close_tests {
    use super::*;

    #[test]
    fn returns_to_standby_and_clears_the_undo_stack() {
        let state = MatchState {
            active: true,
            score_left: 5,
            score_right: 3,
            sets_won_left: 1,
            history_count: 1,
            undo_count: 2,
            name_left: name_bytes("A"),
            name_right: name_bytes("B"),
            best_of: 5,
            ..MatchState::default()
        };

        let next = apply_close(&state);

        let expected = MatchState { active: false, undo_count: 0, ..state.clone() };

        assert_eq!(next, expected);
    }

    #[test]
    fn closes_a_decided_match_the_same_way_as_an_undecided_one() {
        // Unlike Point/Set-server, Close has no freeze guard — it isn't
        // gated on is_match_decided at all.
        let state = MatchState {
            active: true,
            best_of: 3,
            sets_won_left: 2, // decided
            undo_count: 1,
            ..MatchState::default()
        };

        let next = apply_close(&state);

        let expected = MatchState { active: false, undo_count: 0, ..state.clone() };

        assert_eq!(next, expected);
    }
}
