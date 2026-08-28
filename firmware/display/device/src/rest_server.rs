//! HTTP wiring for `docs/protocol.md`'s REST endpoints — the ESP32-specific
//! glue between `esp-idf-svc`'s HTTP server and the host-testable
//! `display-rest` (JSON/validation) + `display-core` (`Application`)
//! crates. Not unit-tested — `device` can't `cargo test` on host (see
//! `docs/slices/02-display-bringup-plan.md`); verified by hand against
//! real hardware, same as `display.rs`.

use display_core::{Application, Command};
use display_rest::request::{PointRequest, RestError, ServerRequest, StartMatchRequest};
use display_rest::response::StateWire;
use embedded_svc::http::server::{Connection, Request};
use embedded_svc::io::Write;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;
use std::sync::{Arc, Mutex};

use crate::display::St7789Display;
use crate::storage::NoopStorage;

type SharedApp = Arc<Mutex<Application<St7789Display<'static>, NoopStorage>>>;

/// Comfortably larger than any real body — the biggest, `StartMatchRequest`
/// (two NAME_LEN=64-byte names plus JSON punctuation/keys), is well under
/// 256 bytes. A sanity ceiling against a malformed/oversized
/// Content-Length, not a tuned limit — but kept modest on purpose: this
/// buffer is stack-allocated inside the httpd worker task's small stack
/// (see `stack_size` below), so it isn't free the way a heap buffer would
/// be.
const MAX_BODY_LEN: usize = 256;

/// A client-caused request failure (bad JSON, failed REST-boundary
/// validation — see `display_rest::request`) — always maps to `400`. A
/// genuine I/O error reading the body is a different, unrecoverable kind
/// of failure and just propagates directly as `anyhow::Error` instead of
/// going through this type — see `read_body`.
struct RequestError(String);

impl From<serde_json::Error> for RequestError {
    fn from(e: serde_json::Error) -> Self {
        RequestError(format!("malformed JSON: {e}"))
    }
}

impl From<RestError> for RequestError {
    fn from(e: RestError) -> Self {
        RequestError(e.to_string())
    }
}

pub fn start(app: SharedApp) -> anyhow::Result<EspHttpServer<'static>> {
    let config = Configuration {
        // The default (6144) crashed with a StoreProhibited panic (stack
        // overflow) handling POST /api/match on real hardware: MatchState
        // is ~1.8KB (dominated by its 200-entry undo_stack) and gets
        // constructed/copied several times through the
        // apply()/Application::handle()/render() call chain, on top of
        // this handler's own MAX_BODY_LEN stack buffer and serde_json's
        // parser. GET /api/state never panicked — it never touches any of
        // that. Bumped generously rather than to the exact minimum that
        // happens to survive today's code.
        stack_size: 16384,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&config)?;

    server.fn_handler("/api/state", Method::Get, {
        let app = app.clone();
        move |request| handle_state(request, &app)
    })?;

    server.fn_handler("/api/point", Method::Post, {
        let app = app.clone();
        move |request| {
            handle_mutation(request, &app, |body| -> Result<Command, RequestError> {
                let req: PointRequest = serde_json::from_slice(body)?;
                Ok(req.into_command()?)
            })
        }
    })?;

    server.fn_handler("/api/server", Method::Post, {
        let app = app.clone();
        move |request| {
            handle_mutation(request, &app, |body| -> Result<Command, RequestError> {
                let req: ServerRequest = serde_json::from_slice(body)?;
                Ok(req.into_command()?)
            })
        }
    })?;

    server.fn_handler("/api/undo", Method::Post, {
        let app = app.clone();
        move |request| handle_mutation(request, &app, |_body| Ok(Command::Undo))
    })?;

    server.fn_handler("/api/close", Method::Post, {
        let app = app.clone();
        move |request| handle_mutation(request, &app, |_body| Ok(Command::Close))
    })?;

    server.fn_handler("/api/match", Method::Post, {
        let app = app.clone();
        move |request| handle_start_match(request, &app)
    })?;

    log::info!("REST server started");
    Ok(server)
}

fn read_body<C>(request: &mut Request<C>, buf: &mut [u8]) -> anyhow::Result<usize>
where
    C: Connection,
    C::Error: std::error::Error + Send + Sync + 'static,
{
    let mut offset = 0;
    loop {
        let n = request.read(&mut buf[offset..])?;
        if n == 0 || offset + n == buf.len() {
            offset += n;
            break;
        }
        offset += n;
    }
    Ok(offset)
}

fn write_json<C>(
    request: Request<C>,
    status: u16,
    body: &impl serde::Serialize,
) -> anyhow::Result<()>
where
    C: Connection,
    C::Error: std::error::Error + Send + Sync + 'static,
{
    let bytes = serde_json::to_vec(body)?;
    let mut response =
        request.into_response(status, None, &[("Content-Type", "application/json")])?;
    response.write_all(&bytes)?;
    Ok(())
}

fn write_error<C>(request: Request<C>, status: u16, message: &str) -> anyhow::Result<()>
where
    C: Connection,
    C::Error: std::error::Error + Send + Sync + 'static,
{
    write_json(request, status, &serde_json::json!({ "error": message }))
}

/// Shared by every endpoint that doesn't need the active-match check
/// `/api/match` alone requires (see `handle_start_match`): read the body,
/// let `parse` turn it into a `Command` (or a validation failure), apply
/// it, and respond with the resulting state — `/api/undo` and
/// `/api/close` just pass a `parse` that ignores the body and always
/// succeeds, since they take none.
fn handle_mutation<C>(
    mut request: Request<C>,
    app: &SharedApp,
    parse: impl FnOnce(&[u8]) -> Result<Command, RequestError>,
) -> anyhow::Result<()>
where
    C: Connection,
    C::Error: std::error::Error + Send + Sync + 'static,
{
    let mut buf = [0u8; MAX_BODY_LEN];
    let n = read_body(&mut request, &mut buf)?;

    let cmd = match parse(&buf[..n]) {
        Ok(cmd) => cmd,
        Err(RequestError(msg)) => return write_error(request, 400, &msg),
    };

    let state_wire = {
        let mut app = app.lock().expect("Application mutex poisoned");
        app.handle(&cmd);
        StateWire::from(app.state())
    };

    write_json(request, 200, &state_wire)
}

/// `POST /api/match` — the one endpoint `docs/protocol.md` documents as
/// erroring (409) rather than no-op'ing, unlike `apply_start_match`'s own
/// silent-no-op-while-active behavior (see
/// `docs/slices/01-backend-logic-requirements.md`). Checked here, before
/// calling `Application::handle()`, rather than by diffing state before
/// and after — cheaper, and avoids a wasted save+render when the request
/// is about to be rejected anyway.
fn handle_start_match<C>(mut request: Request<C>, app: &SharedApp) -> anyhow::Result<()>
where
    C: Connection,
    C::Error: std::error::Error + Send + Sync + 'static,
{
    let mut buf = [0u8; MAX_BODY_LEN];
    let n = read_body(&mut request, &mut buf)?;

    let cmd = match (|| -> Result<Command, RequestError> {
        let req: StartMatchRequest = serde_json::from_slice(&buf[..n])?;
        Ok(req.into_command()?)
    })() {
        Ok(cmd) => cmd,
        Err(RequestError(msg)) => return write_error(request, 400, &msg),
    };

    let mut app_guard = app.lock().expect("Application mutex poisoned");
    if app_guard.state().active {
        drop(app_guard);
        return write_error(request, 409, "a Match is already active — close it first");
    }

    app_guard.handle(&cmd);
    let state_wire = StateWire::from(app_guard.state());
    drop(app_guard);

    write_json(request, 200, &state_wire)
}

fn handle_state<C>(request: Request<C>, app: &SharedApp) -> anyhow::Result<()>
where
    C: Connection,
    C::Error: std::error::Error + Send + Sync + 'static,
{
    let state_wire = {
        let app = app.lock().expect("Application mutex poisoned");
        StateWire::from(app.state())
    };

    write_json(request, 200, &state_wire)
}
