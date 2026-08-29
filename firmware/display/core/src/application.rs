//! The thin, hardware-free shell that wires `apply()` to a `Display` and a
//! `Storage`. The only place ports get called — see
//! `docs/software-design.md`'s "The core is a pure function..." section.

use crate::command::Command;
use crate::match_logic::apply;
use crate::ports::{Display, Storage};
use crate::state::MatchState;

pub struct Application<D: Display, S: Storage> {
    state: MatchState,
    display: D,
    storage: S,
}

impl<D: Display, S: Storage> Application<D, S> {
    /// Resumes from `storage.load()` (or starts at Standby if there's
    /// nothing saved), then renders that state immediately — the resumed
    /// state gets displayed even before any Command arrives.
    pub fn new(mut display: D, storage: S) -> Self {
        let state = storage.load().unwrap_or_default();
        display.render(&state);
        Self {
            state,
            display,
            storage,
        }
    }

    /// Applies `cmd`, then saves and renders the result. The one operation
    /// that touches both ports — everything else here is read-only.
    pub fn handle(&mut self, cmd: &Command) {
        self.state = apply(&self.state, cmd);
        self.storage.save(&self.state);
        self.display.render(&self.state);
    }

    pub fn state(&self) -> &MatchState {
        &self.state
    }

    /// Re-renders the current state as-is — no `Command` applied, storage
    /// untouched. For a `Display` that shows something time-based
    /// alongside the Match (a wall clock, say) and needs to be redrawn
    /// periodically even when nothing else is happening; there's no
    /// "do-nothing" `Command` safe to send through `handle()` for that —
    /// every real `Command` either changes state or has undo-stack side
    /// effects.
    pub fn refresh(&mut self) {
        self.display.render(&self.state);
    }
}
