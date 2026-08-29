//! What a Display should show, computed once from `MatchState` and shared
//! by every concrete `Display` impl — the ST7789 bench display today, the
//! eventual HUB75 panel later. The point: "what facts get shown" is one
//! tested piece of logic every driver reuses, not something each driver
//! re-derives from `MatchState` on its own and risks drifting out of
//! sync with the others. See `docs/architecture.md`'s "Rendering" section
//! for the reference content model (names + serve indicator, current
//! score, sets won, past-Set history) this is built from.
//!
//! Different screens legitimately show different *amounts* of this — a
//! 135x240 bench panel can't fit as many past-Set entries as a 64x64
//! HUB75 grid might (and might not fit some facts at all), and both
//! truncate long names — but which facts exist to show is the same
//! everywhere. That's why `build_view` takes the space budget
//! (`max_name_chars`, `max_history_entries`) as parameters from the
//! caller instead of hard-coding one display's limits: each driver
//! reports its own budget, this crate applies one shared, tested
//! truncation policy to it.
//!
//! Wall-clock time is deliberately not part of this model — it isn't
//! derived from `MatchState` at all (see `device`'s `clock` module for
//! where that actually lives), so it has no business in a crate whose
//! whole job is "a pure function of `MatchState`".
//!
//! No ESP-IDF dependency — `cargo test`-able on host like `core` and
//! `rest` are.

use display_core::{MatchState, Side, is_match_decided};

/// Everything a `Display` needs to draw, already trimmed to fit within
/// the caller's space budget. `None` from `build_view` when there's no
/// Match to show — what a driver does instead (an idle/standby screen)
/// is its own concern, not something this crate has an opinion on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreboardView {
    pub left_name: String,
    pub right_name: String,
    pub server: Side,
    pub score_left: u8,
    pub score_right: u8,
    pub sets_won_left: u8,
    pub sets_won_right: u8,
    /// Completed Sets, most recent last, trimmed to at most
    /// `max_history_entries`. When trimmed, the *oldest* entries are the
    /// ones dropped — the most recent result is what a viewer glancing at
    /// the display mid-match actually wants to see.
    pub history: Vec<(u8, u8)>,
    /// True when `history` omits older entries that don't fit the
    /// caller's budget — enough for a driver to draw an ellipsis/overflow
    /// marker if it wants one.
    pub history_truncated: bool,
    pub decided: bool,
}

/// Truncates to at most `max_chars` *characters*, not bytes — a name can
/// contain multi-byte UTF-8, and cutting mid-character would produce
/// invalid output for the driver to draw.
fn truncate_name(name: &str, max_chars: usize) -> String {
    name.chars().take(max_chars).collect()
}

fn decode_name(buf: &[u8; display_core::NAME_LEN]) -> &str {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    // MatchState's name buffers are only ever written by copy_name, which
    // only ever copies bytes from an already-validated &str (validated at
    // the REST boundary — see display-rest) — invalid UTF-8 here would be
    // an internal invariant violation, not a real-world input to handle
    // gracefully.
    str::from_utf8(&buf[..end]).expect("name buffer is not valid UTF-8 — invariant violated")
}

/// Builds the view for `state`, or `None` if there's no active Match.
/// `max_name_chars` and `max_history_entries` are the caller's own space
/// budget — see the module docs.
pub fn build_view(
    state: &MatchState,
    max_name_chars: usize,
    max_history_entries: usize,
) -> Option<ScoreboardView> {
    if !state.active {
        return None;
    }

    let history_count = state.history_count as usize;
    let full_history = &state.history[..history_count];
    let history_truncated = full_history.len() > max_history_entries;
    let history = full_history[full_history.len().saturating_sub(max_history_entries)..]
        .iter()
        .map(|r| (r.score_left, r.score_right))
        .collect();

    Some(ScoreboardView {
        left_name: truncate_name(decode_name(&state.name_left), max_name_chars),
        right_name: truncate_name(decode_name(&state.name_right), max_name_chars),
        server: state.server,
        score_left: state.score_left,
        score_right: state.score_right,
        sets_won_left: state.sets_won_left,
        sets_won_right: state.sets_won_right,
        history,
        history_truncated,
        decided: is_match_decided(state),
    })
}
