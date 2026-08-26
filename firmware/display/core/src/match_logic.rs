//! The pure `apply(state, command) -> state` entry point — a thin
//! dispatcher, no logic of its own. See `docs/software-design.md`.

use crate::command::Command;
use crate::lifecycle::{apply_close, apply_start_match};
use crate::scoring::apply_point;
use crate::serve::apply_set_server;
use crate::state::MatchState;
use crate::undo::apply_undo;

pub fn apply(state: &MatchState, cmd: &Command) -> MatchState {
    match cmd {
        Command::StartMatch {
            name_left,
            name_right,
            best_of,
        } => apply_start_match(state, name_left, name_right, *best_of),
        Command::Point { side } => apply_point(state, *side),
        Command::Undo => apply_undo(state),
        Command::SetServer { side } => apply_set_server(state, *side),
        Command::Close => apply_close(state),
    }
}
