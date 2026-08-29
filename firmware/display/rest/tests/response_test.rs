//! `MatchState` -> wire JSON, matching `docs/protocol.md`'s state-push
//! shape exactly (field names, the `NO_MATCH` shape, computed fields).

use display_core::{MatchState, NAME_LEN, SetResult, Side};
use display_rest::response::{SetResultWire, StateWire};

fn name_bytes(name: &str) -> [u8; NAME_LEN] {
    let mut buf = [0u8; NAME_LEN];
    buf[..name.len()].copy_from_slice(name.as_bytes());
    buf
}

/// Pins the actual wire *shape* (key names, the collapsed `NO_MATCH` form)
/// by round-tripping through `serde_json::Value` against a fully-specified
/// expected document — the rest of this file asserts on `StateWire` values
/// directly, which doesn't exercise the `#[serde(...)]` attributes at all.
mod wire_shape {
    use super::*;

    #[test]
    fn no_match_serializes_to_just_active_false() {
        let state = MatchState::default(); // active: false

        let value = serde_json::to_value(StateWire::from(&state)).unwrap();

        assert_eq!(value, serde_json::json!({ "active": false }));
    }

    #[test]
    fn active_match_serializes_with_every_documented_field() {
        let state = MatchState {
            active: true,
            name_left: name_bytes("ALEX"),
            name_right: name_bytes("JORDAN"),
            best_of: 5,
            sets_won_left: 1,
            sets_won_right: 0,
            score_left: 7,
            score_right: 5,
            server: Side::Left,
            history: [SetResult {
                score_left: 11,
                score_right: 7,
            }; display_core::MAX_SETS as usize],
            history_count: 1,
            undo_count: 1,
            ..MatchState::default()
        };

        let value = serde_json::to_value(StateWire::from(&state)).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "active": true,
                "nameLeft": "ALEX", "nameRight": "JORDAN",
                "bestOf": 5,
                "setsWonLeft": 1, "setsWonRight": 0,
                "scoreLeft": 7, "scoreRight": 5,
                "server": "left",
                "history": [ { "scoreLeft": 11, "scoreRight": 7 } ],
                "decided": false,
                "canUndo": true
            })
        );
    }
}

/// `StateWire::Active`'s fields at their "just started" defaults — every
/// test below builds its expected value by naming only the fields it
/// cares about, same shape as `..MatchState::default()` elsewhere, but
/// spelled out in full since Rust's functional-update syntax doesn't
/// apply to enum struct variants (E0436).
mod match_state_to_state_wire {
    use super::*;

    fn active_default() -> MatchState {
        MatchState {
            active: true,
            ..MatchState::default()
        }
    }

    #[test]
    fn server_right_is_lowercase_right_not_a_debug_repr() {
        let state = MatchState {
            server: Side::Right,
            ..active_default()
        };

        assert_eq!(
            StateWire::from(&state),
            StateWire::Active {
                active: true,
                name_left: String::new(),
                name_right: String::new(),
                best_of: 0,
                sets_won_left: 0,
                sets_won_right: 0,
                score_left: 0,
                score_right: 0,
                server: "right".to_string(),
                history: Vec::new(),
                decided: false,
                can_undo: false,
            }
        );
    }

    #[test]
    fn history_is_trimmed_to_history_count_not_the_full_backing_array() {
        let mut history = [SetResult::default(); display_core::MAX_SETS as usize];
        history[0] = SetResult {
            score_left: 11,
            score_right: 9,
        };
        history[1] = SetResult {
            score_left: 8,
            score_right: 11,
        };
        let state = MatchState {
            history,
            history_count: 2,
            ..active_default()
        };

        assert_eq!(
            StateWire::from(&state),
            StateWire::Active {
                active: true,
                name_left: String::new(),
                name_right: String::new(),
                best_of: 0,
                sets_won_left: 0,
                sets_won_right: 0,
                score_left: 0,
                score_right: 0,
                server: "left".to_string(),
                history: vec![
                    SetResultWire {
                        score_left: 11,
                        score_right: 9
                    },
                    SetResultWire {
                        score_left: 8,
                        score_right: 11
                    },
                ],
                decided: false,
                can_undo: false,
            }
        );
    }

    #[test]
    fn decided_reflects_a_won_match_not_just_a_won_set() {
        // best_of 3, 2 Sets won left — majority reached, Match is decided.
        let state = MatchState {
            best_of: 3,
            sets_won_left: 2,
            ..active_default()
        };

        assert_eq!(
            StateWire::from(&state),
            StateWire::Active {
                active: true,
                name_left: String::new(),
                name_right: String::new(),
                best_of: 3,
                sets_won_left: 2,
                sets_won_right: 0,
                score_left: 0,
                score_right: 0,
                server: "left".to_string(),
                history: Vec::new(),
                decided: true,
                can_undo: false,
            }
        );
    }

    #[test]
    fn can_undo_is_true_when_the_undo_stack_is_non_empty() {
        let state = MatchState {
            undo_count: 1,
            ..active_default()
        };

        assert_eq!(
            StateWire::from(&state),
            StateWire::Active {
                active: true,
                name_left: String::new(),
                name_right: String::new(),
                best_of: 0,
                sets_won_left: 0,
                sets_won_right: 0,
                score_left: 0,
                score_right: 0,
                server: "left".to_string(),
                history: Vec::new(),
                decided: false,
                can_undo: true,
            }
        );
    }

    #[test]
    fn name_bytes_are_decoded_as_utf8_and_trimmed_of_zero_padding() {
        let state = MatchState {
            name_left: name_bytes("Zoë"), // multi-byte UTF-8, well under NAME_LEN
            ..active_default()
        };

        assert_eq!(
            StateWire::from(&state),
            StateWire::Active {
                active: true,
                name_left: "Zoë".to_string(),
                name_right: String::new(),
                best_of: 0,
                sets_won_left: 0,
                sets_won_right: 0,
                score_left: 0,
                score_right: 0,
                server: "left".to_string(),
                history: Vec::new(),
                decided: false,
                can_undo: false,
            }
        );
    }
}
