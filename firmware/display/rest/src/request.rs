//! Incoming JSON request bodies -> `Command`, including the REST-boundary
//! validation `display_core` explicitly assumes has already happened —
//! name byte length, `best_of` odd/`<= MAX_SETS`, `side` parsing. See
//! `docs/slices/01-backend-logic-requirements.md`'s "Explicitly out of
//! scope" and `docs/protocol.md`'s endpoint table.

use display_core::{Command, MAX_SETS, NAME_LEN, Side};
use serde::Deserialize;
use std::fmt;

/// Why a request body failed REST-boundary validation. Distinct from a
/// JSON parse failure (malformed body) — that's `device`'s concern at the
/// `serde_json::from_slice` call site, before any of these types exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestError {
    /// `side` wasn't `"left"` or `"right"`.
    InvalidSide,
    /// A name's UTF-8 *byte* length (not char count — see `NAME_LEN`'s doc
    /// comment) exceeded `NAME_LEN`.
    NameTooLong { field: &'static str },
    /// `bestOf` wasn't odd, was zero, or exceeded `MAX_SETS`.
    InvalidBestOf,
}

impl fmt::Display for RestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestError::InvalidSide => write!(f, "side must be \"left\" or \"right\""),
            RestError::NameTooLong { field } => write!(f, "{field} exceeds {NAME_LEN} bytes"),
            RestError::InvalidBestOf => write!(f, "bestOf must be odd and at most {MAX_SETS}"),
        }
    }
}

fn parse_side(side: &str) -> Result<Side, RestError> {
    match side {
        "left" => Ok(Side::Left),
        "right" => Ok(Side::Right),
        _ => Err(RestError::InvalidSide),
    }
}

fn validate_name(name: &str, field: &'static str) -> Result<(), RestError> {
    if name.len() > NAME_LEN {
        Err(RestError::NameTooLong { field })
    } else {
        Ok(())
    }
}

fn validate_best_of(best_of: u8) -> Result<(), RestError> {
    if best_of == 0 || best_of.is_multiple_of(2) || best_of > MAX_SETS {
        Err(RestError::InvalidBestOf)
    } else {
        Ok(())
    }
}

/// `POST /api/point` body.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PointRequest {
    pub side: String,
}

impl PointRequest {
    pub fn into_command(&self) -> Result<Command, RestError> {
        Ok(Command::Point {
            side: parse_side(&self.side)?,
        })
    }
}

/// `POST /api/server` body.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ServerRequest {
    pub side: String,
}

impl ServerRequest {
    pub fn into_command(&self) -> Result<Command, RestError> {
        Ok(Command::SetServer {
            side: parse_side(&self.side)?,
        })
    }
}

/// `POST /api/match` body.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StartMatchRequest {
    pub name_left: String,
    pub name_right: String,
    pub best_of: u8,
}

impl StartMatchRequest {
    pub fn into_command(&self) -> Result<Command, RestError> {
        validate_name(&self.name_left, "nameLeft")?;
        validate_name(&self.name_right, "nameRight")?;
        validate_best_of(self.best_of)?;
        Ok(Command::StartMatch {
            name_left: self.name_left.clone(),
            name_right: self.name_right.clone(),
            best_of: self.best_of,
        })
    }
}
