//! The Standby <-> In-Match transition — `Start-match` and `Close`. See
//! `docs/architecture.md`'s "Match lifecycle".

use crate::state::{MatchState, NAME_LEN};

/// Copies `name`'s UTF-8 bytes into a `NAME_LEN`-sized buffer. Assumes
/// `name`'s byte length fits within `NAME_LEN` — that's a REST-boundary
/// validation concern (slice 3), not this crate's job to enforce; see
/// `NAME_LEN`'s doc comment.
fn copy_name(name: &str) -> [u8; NAME_LEN] {
    let mut buf = [0u8; NAME_LEN];
    let bytes = name.as_bytes();
    buf[..bytes.len()].copy_from_slice(bytes);
    buf
}

/// Starts a Match from Standby: sets names/`best_of`, resets everything
/// else (score, history, undo, server, Set-win counts) to fresh defaults,
/// transitions to In-Match. A no-op (state unchanged) if a Match is
/// already active — never silently discards one, matching the general
/// "doesn't apply here" identity-transform rule other commands follow.
pub fn apply_start_match(state: &MatchState, name_left: &str, name_right: &str, best_of: u8) -> MatchState {
    if state.active {
        return state.clone();
    }

    MatchState {
        active: true,
        name_left: copy_name(name_left),
        name_right: copy_name(name_right),
        best_of,
        ..MatchState::default()
    }
}

/// Returns to Standby and clears the undo stack, regardless of whether the
/// Match was decided — no freeze guard, unlike `Point`/`Set-server`.
/// Deliberately leaves score/history/`sets_won_*`/names untouched (not
/// reset to defaults): the next `Start-match` will overwrite them anyway,
/// and there's no correctness reason to wipe them sooner.
pub fn apply_close(state: &MatchState) -> MatchState {
    let mut next = state.clone();
    next.active = false;
    next.undo_count = 0;
    next
}
