//! The `Command` type flowing into the state-manipulation core's
//! `apply()`. See `docs/slices/01-backend-logic-requirements.md`.

use crate::state::Side;

/// A single value type, not a trait with a method per action —
/// composes with a queue directly (see `docs/software-design.md`'s
/// rationale). `Unlock` is cut from MVP scope — see
/// `docs/slices/01-backend-logic-requirements.md`'s "Explicitly out of
/// scope." Each variant carries only the data that command needs — no
/// fields left irrelevant/unused depending on which variant it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    StartMatch { name_left: String, name_right: String, best_of: u8 },
    Point { side: Side },
    Undo,
    SetServer { side: Side },
    Close,
}
