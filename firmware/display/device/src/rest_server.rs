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
/// 256 bytes. Enforced against the request's declared Content-Length
/// *before* reading any of the body (see `read_body`), so a body over
/// this cap is rejected without ever partially consuming it — no leftover
/// unread bytes to desync a reused (keep-alive) connection's next
/// request.
const MAX_BODY_LEN: usize = 256;

/// The minimal phone control page — Checkpoint G. A single self-contained
/// file (inline CSS/JS, no external requests) baked into the binary at
/// compile time; no LittleFS needed to serve it. See
/// `docs/slices/02-display-bringup-plan.md`'s Checkpoint G for why this
/// approach was chosen over LittleFS-backed static files (`protocol.md`'s
/// original framing) or hosting the page elsewhere.
const CONTROL_PAGE: &str = include_str!("../static/index.html");

/// A request failure that gets reported back to the client as `400` —
/// bad JSON, failed REST-boundary validation (see `display_rest::request`),
/// an over-cap or unreadable body. Genuine failures elsewhere (writing the
/// response itself, the JSON encoder) stay `anyhow::Error` and propagate
/// as a hard error instead — there's no client left to usefully report
/// those to at that point.
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

    server.fn_handler("/", Method::Get, |request| -> anyhow::Result<()> {
        let mut response =
            request.into_response(200, None, &[("Content-Type", "text/html; charset=utf-8")])?;
        response.write_all(CONTROL_PAGE.as_bytes())?;
        Ok(())
    })?;

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

/// Rejects an over-cap body by its declared Content-Length, before
/// reading any of it — reading up to a fixed buffer size and stopping
/// there (the previous approach) can leave bytes the client already sent
/// unread on the connection, corrupting the next request on a reused
/// (keep-alive) connection. A client that omits Content-Length on a
/// request with a body (chunked transfer encoding, or simply malformed)
/// isn't handled specially — this protocol's own clients (curl, fetch(),
/// any JSON HTTP library) always send it for a plain POST body, and
/// protocol.md already scopes out REST-boundary robustness against
/// malicious/malformed clients for this bring-up pass.
fn read_body<C>(request: &mut Request<C>) -> Result<Vec<u8>, RequestError>
where
    C: Connection,
    C::Error: std::error::Error + Send + Sync + 'static,
{
    let content_length: usize = request
        .header("Content-Length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    if content_length > MAX_BODY_LEN {
        return Err(RequestError(format!(
            "request body exceeds {MAX_BODY_LEN} bytes"
        )));
    }

    let mut body = vec![0u8; content_length];
    let mut offset = 0;
    while offset < body.len() {
        let n = request
            .read(&mut body[offset..])
            .map_err(|e| RequestError(format!("failed to read request body: {e}")))?;
        if n == 0 {
            // Connection closed before the declared Content-Length was
            // fully sent. Don't treat as an I/O error — truncated JSON
            // fails to parse on its own and reports as an ordinary 400
            // through the same path as any other malformed body.
            break;
        }
        offset += n;
    }
    body.truncate(offset);
    Ok(body)
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
    let cmd = match read_body(&mut request).and_then(|body| parse(&body)) {
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
    let cmd = match read_body(&mut request).and_then(|body| {
        let req: StartMatchRequest = serde_json::from_slice(&body)?;
        Ok(req.into_command()?)
    }) {
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
