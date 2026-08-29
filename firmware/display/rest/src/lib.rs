//! Wire-format layer for the REST/HTTP API described in `docs/protocol.md`
//! — JSON (de)serialization and the REST-boundary input validation
//! `display_core::apply()` explicitly assumes has already happened (see
//! `docs/slices/01-backend-logic-requirements.md`). Depends only on
//! `display-core` + `serde`, no ESP-IDF — host-testable with `cargo test`
//! like `core` is. `device` wires this to actual HTTP handlers, same
//! hexagonal-adapter shape as the `Display`/`Storage` ports.

pub mod request;
pub mod response;
