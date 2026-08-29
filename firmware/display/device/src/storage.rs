//! `display_core::Storage` implemented as a no-op. Persistence (LittleFS)
//! is explicitly deferred for this bring-up pass — see
//! `docs/slices/02-display-bringup-plan.md`'s scope decisions. State
//! lives in memory only and resets on reboot.

use display_core::{MatchState, Storage};

pub struct NoopStorage;

impl Storage for NoopStorage {
    fn save(&mut self, _state: &MatchState) {}
    fn load(&self) -> Option<MatchState> {
        None
    }
}
