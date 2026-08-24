//! Pure state-manipulation core for the Display firmware — no hardware, no
//! I/O, no OS. See `docs/software-design.md` and
//! `docs/adr/0006-rust-for-lib-core.md`.

pub mod command;
pub mod scoring;
pub mod serve;
pub mod set_progression;
pub mod state;
pub mod undo;

pub use command::*;
pub use scoring::*;
pub use serve::*;
pub use set_progression::*;
pub use state::*;
pub use undo::*;
