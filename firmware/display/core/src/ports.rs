//! Abstract Display / Storage interfaces — see ADR-0004 and
//! `docs/software-design.md`'s "Ports" section. Real implementations
//! (a HUB75 or ST7789 driver, LittleFS) and fakes both just implement
//! these two traits.

use crate::state::MatchState;

pub trait Display {
    fn render(&mut self, state: &MatchState);
}

pub trait Storage {
    fn save(&mut self, state: &MatchState);
    fn load(&self) -> Option<MatchState>;
}
