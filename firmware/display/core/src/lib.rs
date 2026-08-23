//! Pure state-manipulation core for the Display firmware — no hardware, no
//! I/O, no OS. See `docs/software-design.md` and
//! `docs/adr/0006-rust-for-lib-core.md`.

pub mod command;
pub mod state;

pub use command::*;
pub use state::*;
