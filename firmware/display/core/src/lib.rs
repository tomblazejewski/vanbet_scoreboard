//! Pure state-manipulation core for the Display firmware — no hardware, no
//! I/O, no OS. See `docs/software-design.md` and
//! `docs/adr/0006-rust-for-lib-core.md`.

pub mod application;
pub mod command;
pub mod lifecycle;
pub mod match_logic;
pub mod ports;
pub mod scoring;
pub mod serve;
pub mod set_progression;
pub mod state;
pub mod undo;

pub use application::*;
pub use command::*;
pub use lifecycle::*;
pub use match_logic::*;
pub use ports::*;
pub use scoring::*;
pub use serve::*;
pub use set_progression::*;
pub use state::*;
pub use undo::*;
