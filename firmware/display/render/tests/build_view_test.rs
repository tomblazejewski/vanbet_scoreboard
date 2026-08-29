//! `MatchState` -> `ScoreboardView`: the shared content model every
//! `Display` driver draws from. See the crate's module docs for why
//! truncation limits are parameters rather than hard-coded.

use display_core::{MatchState, NAME_LEN, SetResult, Side};
use display_render::{ScoreboardView, build_view};

fn name_bytes(name: &str) -> [u8; NAME_LEN] {
    let mut buf = [0u8; NAME_LEN];
    buf[..name.len()].copy_from_slice(name.as_bytes());
    buf
}

fn active_default() -> MatchState {
    MatchState {
        active: true,
        ..MatchState::default()
    }
}

#[test]
fn no_match_produces_no_view() {
    let state = MatchState::default(); // active: false

    assert_eq!(build_view(&state, 20, 5), None);
}

#[test]
fn a_fully_specified_active_match_produces_the_matching_view() {
    let state = MatchState {
        name_left: name_bytes("ALEX"),
        name_right: name_bytes("JORDAN"),
        best_of: 5,
        sets_won_left: 1,
        sets_won_right: 0,
        score_left: 7,
        score_right: 5,
        server: Side::Left,
        history: {
            let mut h = [SetResult::default(); display_core::MAX_SETS as usize];
            h[0] = SetResult {
                score_left: 11,
                score_right: 7,
            };
            h
        },
        history_count: 1,
        ..active_default()
    };

    assert_eq!(
        build_view(&state, 20, 5),
        Some(ScoreboardView {
            left_name: "ALEX".to_string(),
            right_name: "JORDAN".to_string(),
            server: Side::Left,
            score_left: 7,
            score_right: 5,
            sets_won_left: 1,
            sets_won_right: 0,
            history: vec![(11, 7)],
            history_truncated: false,
            decided: false,
        })
    );
}

#[test]
fn name_longer_than_the_budget_is_truncated_to_exactly_that_many_chars() {
    let state = MatchState {
        name_left: name_bytes("ALEXANDRIA"),
        ..active_default()
    };

    let view = build_view(&state, 5, 5).unwrap();

    assert_eq!(view.left_name, "ALEXA");
}

#[test]
fn name_truncation_cuts_on_a_char_boundary_not_a_byte_boundary() {
    // Each 'Z' here is 'Ż' (U+017B, 2 UTF-8 bytes) — cutting by byte count
    // instead of char count would either panic (str::from_utf8 on a
    // split multi-byte sequence) or silently corrupt the string.
    let state = MatchState {
        name_left: name_bytes("ŻŻŻŻŻŻ"),
        ..active_default()
    };

    let view = build_view(&state, 3, 5).unwrap();

    assert_eq!(view.left_name, "ŻŻŻ");
}

#[test]
fn name_within_the_budget_is_unchanged() {
    let state = MatchState {
        name_left: name_bytes("AL"),
        ..active_default()
    };

    let view = build_view(&state, 20, 5).unwrap();

    assert_eq!(view.left_name, "AL");
}

#[test]
fn history_within_the_budget_is_kept_in_full_oldest_first() {
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

    let view = build_view(&state, 20, 5).unwrap();

    assert_eq!(view.history, vec![(11, 9), (8, 11)]);
    assert!(!view.history_truncated);
}

#[test]
fn history_over_the_budget_keeps_only_the_most_recent_entries() {
    let mut history = [SetResult::default(); display_core::MAX_SETS as usize];
    history[0] = SetResult {
        score_left: 11,
        score_right: 1,
    }; // oldest — should be dropped
    history[1] = SetResult {
        score_left: 11,
        score_right: 2,
    };
    history[2] = SetResult {
        score_left: 11,
        score_right: 3,
    }; // most recent
    let state = MatchState {
        history,
        history_count: 3,
        ..active_default()
    };

    let view = build_view(&state, 20, 2).unwrap();

    assert_eq!(view.history, vec![(11, 2), (11, 3)]);
    assert!(view.history_truncated);
}

#[test]
fn decided_reflects_a_won_match() {
    // best_of 3, 2 Sets won left — majority reached, Match is decided.
    let state = MatchState {
        best_of: 3,
        sets_won_left: 2,
        ..active_default()
    };

    let view = build_view(&state, 20, 5).unwrap();

    assert!(view.decided);
}
