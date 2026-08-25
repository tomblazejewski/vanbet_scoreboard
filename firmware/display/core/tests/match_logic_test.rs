//! Pins `match_logic::apply`'s dispatch: each `Command` variant routes to
//! its own module's function. Doesn't re-test those functions' own logic
//! (covered in their own test files) — just that the match arms wire up
//! to the right one, so a copy-paste mistake in the dispatcher itself
//! would be caught.

use display_core::{
    Command, MatchState, Side, apply, apply_close, apply_point, apply_set_server,
    apply_start_match, apply_undo,
};

mod apply_dispatch_tests {
    use super::*;

    #[test]
    fn routes_start_match_to_apply_start_match() {
        let state = MatchState::default();
        let cmd = Command::StartMatch {
            name_left: "Alice".to_string(),
            name_right: "Bob".to_string(),
            best_of: 5,
        };

        assert_eq!(apply(&state, &cmd), apply_start_match(&state, "Alice", "Bob", 5));
    }

    #[test]
    fn routes_point_to_apply_point() {
        let state = MatchState { active: true, ..MatchState::default() };
        let cmd = Command::Point { side: Side::Right };

        assert_eq!(apply(&state, &cmd), apply_point(&state, Side::Right));
    }

    #[test]
    fn routes_undo_to_apply_undo() {
        let state =
            MatchState { active: true, score_left: 1, undo_count: 1, ..MatchState::default() };
        let cmd = Command::Undo;

        assert_eq!(apply(&state, &cmd), apply_undo(&state));
    }

    #[test]
    fn routes_set_server_to_apply_set_server() {
        let state = MatchState { active: true, ..MatchState::default() };
        let cmd = Command::SetServer { side: Side::Right };

        assert_eq!(apply(&state, &cmd), apply_set_server(&state, Side::Right));
    }

    #[test]
    fn routes_close_to_apply_close() {
        let state = MatchState { active: true, ..MatchState::default() };
        let cmd = Command::Close;

        assert_eq!(apply(&state, &cmd), apply_close(&state));
    }
}
