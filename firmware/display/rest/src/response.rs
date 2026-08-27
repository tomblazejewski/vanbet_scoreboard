//! `MatchState` -> the wire JSON shape from `docs/protocol.md`'s state
//! push — used for the WebSocket push there (not built yet — see the
//! bring-up plan's "explicitly out of scope"), and for every REST
//! endpoint's response body here.

use display_core::{MatchState, NAME_LEN, Side, is_match_decided};
use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetResultWire {
    pub score_left: u8,
    pub score_right: u8,
}

/// `{"active": false}` (all other fields omitted) for `NO_MATCH`, or the
/// full state otherwise — two genuinely different JSON shapes, not one
/// shape with blanks. `#[serde(untagged)]` picks whichever variant this
/// was built as, with no added discriminant field.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum StateWire {
    Inactive {
        active: bool,
    },
    #[serde(rename_all = "camelCase")]
    Active {
        active: bool,
        name_left: String,
        name_right: String,
        best_of: u8,
        sets_won_left: u8,
        sets_won_right: u8,
        score_left: u8,
        score_right: u8,
        server: String,
        history: Vec<SetResultWire>,
        decided: bool,
        can_undo: bool,
    },
}

/// Decodes a `copy_name`-written buffer: valid UTF-8 bytes followed by
/// zero padding. Panics on invalid UTF-8 — that would mean something
/// other than `copy_name` (which only ever copies bytes from an
/// already-validated `&str`) wrote this buffer, an internal invariant
/// violation rather than a user-facing error to report.
fn decode_name(buf: &[u8; NAME_LEN]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(NAME_LEN);
    str::from_utf8(&buf[..end])
        .expect("name buffer is not valid UTF-8 — invariant violated")
        .to_string()
}

fn side_to_wire(side: Side) -> &'static str {
    match side {
        Side::Left => "left",
        Side::Right => "right",
    }
}

impl From<&MatchState> for StateWire {
    fn from(state: &MatchState) -> Self {
        if !state.active {
            return StateWire::Inactive { active: false };
        }

        let history_count = state.history_count as usize;
        let history = state.history[..history_count]
            .iter()
            .map(|r| SetResultWire {
                score_left: r.score_left,
                score_right: r.score_right,
            })
            .collect();

        StateWire::Active {
            active: true,
            name_left: decode_name(&state.name_left),
            name_right: decode_name(&state.name_right),
            best_of: state.best_of,
            sets_won_left: state.sets_won_left,
            sets_won_right: state.sets_won_right,
            score_left: state.score_left,
            score_right: state.score_right,
            server: side_to_wire(state.server).to_string(),
            history,
            decided: is_match_decided(state),
            can_undo: state.undo_count > 0,
        }
    }
}
