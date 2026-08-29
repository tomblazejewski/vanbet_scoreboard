# Slice 2 (+ slice 3's REST layer): ESP32 bring-up plan

Grilled in chat, lighter-weight than [01-backend-logic-requirements.md](01-backend-logic-requirements.md)'s
full behavior grilling — this records the checkpoint plan and the
decisions needed to start bring-up, not a fully-resolved behavior spec for
either slice. Slice 3's own requirements (Controller design, input
validation, error responses, auth-adjacent questions) are still open and
need their own grilling later; this doc only covers the REST endpoints
needed to drive `apply()` from a phone during Display bring-up.

**This is a bring-up runbook, not a slice requirements doc** — unlike
`01-backend-logic-requirements.md`, it deliberately names concrete
tooling/hardware (`espup`, `esp-idf-template`, the LILYGO TTGO T-Display)
and reports what was actually installed/built, the same "status fact"
carve-out `slices.md` gets under CLAUDE.md's "Slicing conventions". That's
intentional here: this pass *is* real-hardware bring-up, so the toolchain
isn't a swappable implementation detail to keep out of the doc — it's the
subject of the doc.

## Why slices 2 and 3 are being built together here

Both need the same underlying infrastructure on real hardware — a
Rust-on-ESP32 toolchain, WiFi, and the `ports.rs`/`application.rs` shell
slice 1 deliberately deferred. Building that infrastructure once and
layering both slices' actual behavior on top is more efficient than
standing it up twice. The REST endpoints (slice 3) exist here specifically
*to* exercise the Display port (slice 2) from a phone — they're not a
replacement for slice 3's own full grilling later.

## Hardware

LILYGO TTGO T-Display — ESP32 (chip: **ESP32-D0WDQ6**, Xtensa, the classic
dual-core ESP32) with an integrated 1.14" ST7789V IPS panel, 135×240. This
is exactly the bench display [slices.md](../slices.md) describes for
slice 2 — the ST7789 driver implements the same `Display` trait the
eventual HUB75 panel driver will, so this is a real, reusable
implementation of the port, not a throwaway stand-in.

## Scope decisions made so far

- **Persistence (LittleFS) is deferred.** State lives in memory only for
  this pass and resets on reboot. Architecture.md's resume-on-boot
  behavior isn't implemented yet — revisit once this bring-up works.
- **Display renders for real**, not a no-op stub — draws to the ST7789 via
  an SPI TFT driver crate (`mipidsi`) + `embedded-graphics`. First cut is
  plain text (at minimum the live score), not the final layout from
  architecture.md's "Rendering" section (that needs its own design pass
  once this is working).
- **No Controller/ESP-NOW in this pass.** The Controller's design (and
  whether it's even dedicated hardware — still an open question from
  earlier grilling) is untouched. Only the phone/HTTP half of
  [protocol.md](../protocol.md) is being built.
- **REST endpoints match `protocol.md` exactly** (`POST /api/match`,
  `/api/point`, `/api/undo`, `/api/server`, `/api/close`,
  `GET /api/state`) — not a shorthand/approximation, so this is directly
  reusable once slice 3 is properly grilled, not a spike to discard.
- **No auth, no WebSocket yet** — both already documented in `protocol.md`
  as either permanent (no auth, by design) or deferred-but-anticipated
  (the WebSocket push; `protocol.md` explicitly calls out that the HTTP
  endpoints work standalone for "scripting/testing").

## Checkpoints

**A. Toolchain bring-up — done.** Installed `espup` (Xtensa fork —
`ESP32-D0WDQ6` isn't on stock `rustup`), scaffolded a minimal
`esp-idf-template` "hello world" at `firmware/display/device`, flashed it,
confirmed "Hello, world!" over serial. Three real issues hit and fixed
along the way (see the Checkpoint A commit for full detail): a Windows
path-length limit (twice — build output, then ESP-IDF's own source
checkout, both fixed by relocating them to short paths since this repo
sits deep under a OneDrive-synced folder), and unreliable large-file
downloads from GitHub releases (worked around with resumable `curl`).

Flashing setup notes for next time (all local-machine config, not
committed): the board's CH9102 USB-serial chip needs its Windows driver
installed from WCH directly (not Chocolatey — no admin available in this
environment); `espflash` needs `--non-interactive` or it silently hangs
trying to prompt for port selection with no TTY available; the port
itself (`COM6` here) is set via the `ESPFLASH_PORT` env var locally
rather than hardcoded into the committed `.cargo/config.toml`, since it's
specific to this machine/USB port, not portable. Same treatment for the
Windows path-length workaround: set `CARGO_TARGET_DIR` to a short local
path (e.g. `C:/esp-build/device`) as a local env var rather than
committing `target-dir` into `.cargo/config.toml`, since a hardcoded
Windows-absolute path there would break non-Windows/CI builds outright.

**B. `ports.rs` + `application.rs` — done.** Added to the existing
`firmware/display/core` crate — hardware-free, `cargo test`-able on host,
same TDD approach as slice 1. Defines the `Display`/`Storage` traits (from
[software-design.md](../software-design.md)'s "Ports" section) and the
`Application` shell that calls `apply()` + `storage.save()` +
`display.render()`.

**C. WiFi station connection — done.** Joins the network and logs the IP
over serial (`BlockingWifi`, connect-once-and-log — no retry/backoff yet,
that's real always-on-deployment behavior to design deliberately later,
not to guess at now). Credentials live in `src/secrets.rs` (gitignored,
never committed — copy it from the committed `secrets.rs.example` and
fill in real values yourself). `build.rs` stages whichever of the two
exists into `OUT_DIR`, and `main.rs` pulls it in with
`include!(concat!(env!("OUT_DIR"), "/secrets.rs"))` rather than a plain
`mod secrets;` — so a fresh clone or CI compiles against the harmless
placeholder with zero setup, real local credentials are never read or
touched by anything except this copy step, and the file's existence is
never required for anything other than an actual build. Two more
"obvious" fixes were tried first and both broke `cargo fmt --all
--check` (this repo's first-ever fmt check, which is what surfaced all
of this): a Cargo feature switching `#[cfg]`-gated `#[path]` targets
(rustfmt doesn't evaluate `#[cfg(feature = ...)]` the way rustc does
when deciding which `#[path]`-gated module to format), and seeding
`src/secrets.rs` directly from `build.rs` (`cargo fmt` never runs build
scripts, so the file still didn't exist when rustfmt went looking for
it). `include!()` sidesteps both — unlike `mod name;`, rustfmt never
resolves or requires the existence of an `include!()`'d path.

**D. Minimal ST7789 rendering — done.** A concrete `Display` impl (real
`mipidsi` + `embedded-graphics`, `firmware/display/device/src/display.rs`)
draws plain text (the live score) to the actual TTGO screen, confirmed on
hardware. Three real bugs found and fixed along the way — see the
Checkpoint D commit for full detail: the wrong SPI peripheral (SPI2
produced nothing; SPI3 worked), a Rust ownership/Drop bug (the display
value was scoped to a match arm and got dropped — pins released — the
instant that block ended, causing "renders once, then goes dark"), and an
offset-support gap in the `st7789` crate (tried first; it sends raw
address-window coordinates with no offset mechanism, so it couldn't
express this panel's 135x240-on-a-240x240-controller quirk) resolved by
switching to `mipidsi`, which supports an explicit `display_offset()` and
auto-swaps width/height per rotation.

**E. The REST layer — done, and F folded into it.** `protocol.md`'s six
endpoints, live via `esp-idf-svc`'s `EspHttpServer`, each parsing JSON
into a `Command` and calling `Application::handle()` — confirmed against
real hardware with `curl` for every endpoint plus the documented error
cases (`409` on `/api/match` while already active, `400` on invalid
`side`/`bestOf`/malformed JSON). Wiring the REST layer to one live
`Application` instance *is* Checkpoint F's "wire them together" — there
wasn't a separate step once E's device-side code existed, so F is done as
a consequence rather than its own pass.

Split across two crates, continuing the pattern Checkpoint D validated
(hardware-agnostic logic vs. ESP32-specific glue): a new
`firmware/display/rest` crate (JSON wire DTOs, REST-boundary input
validation `display_core::apply()` explicitly assumes has already
happened — `bestOf` odd/`<=MAX_SETS`, name byte length — no ESP-IDF
dependency, `cargo test`-able on host, 21 tests) plus thin,
untested-on-host `device`-crate glue (`rest_server.rs`) registering the
actual HTTP handlers behind an
`Arc<Mutex<Application<St7789Display, NoopStorage>>>`.

One real bug found on hardware: the httpd worker task's default 6KB stack
overflowed (`Guru Meditation Error: StoreProhibited`) handling
`POST /api/match` — `MatchState` is ~1.8KB (dominated by its 200-entry
undo stack) and gets constructed/copied several times through
`apply()`/`Application::handle()`/`render()`, on top of the request
handler's own body buffer and `serde_json`'s parser. `GET /api/state`
never panicked, since it never touches any of that. Fixed by raising
`Configuration::stack_size` to 16KB and shrinking the body buffer from a
needlessly generous 1024 bytes to 256 (the largest real body,
`StartMatchRequest`, is under 200).

**H. Full-content rendering + a persistent clock — done**, confirmed on
real hardware. Lettered H rather than G since
[PR #6](https://github.com/tomblazejewski/vanbet_scoreboard/pull/6)
(the minimal phone control page, Checkpoint G) was still unmerged when
this was grilled — whichever of the two lands second should renumber to
close the gap.

Grilled from scratch after an initial unreviewed implementation attempt
was reverted at the user's request — see this checkpoint's git history
for the full back-and-forth (mockup iterations, the two-column layout
decision, the clock scope questions) rather than just the resolved
answers below:

- **Content**: names (server-marked, truncated), current score, sets
  won, past-Set history (most recent kept when trimmed — not oldest),
  a decided indicator, and a persistent wall-clock time. Computed by a
  new `firmware/display/render` crate (`ScoreboardView`/`build_view`)
  shared by every `Display` driver — same pattern as `display-rest`:
  no ESP-IDF dependency, `cargo test`-able on host, 8 tests. Each
  driver reports its own space budget (`max_name_chars`,
  `max_history_entries`) rather than the crate hard-coding one
  display's limits — the eventual HUB75 panel (64x64, square, far
  fewer pixels) will need a much smaller budget and likely won't fit
  every fact at once (the clock especially), which is a per-display
  layout decision, not a content gap.
- **ST7789 layout**: two columns, not stacked — this screen is
  landscape (240x135), and a single-column layout wastes the width.
  Big score dominant on the left; names/server/sets/history/decided in
  a narrower right column; the clock in the top-right corner,
  persistent across both Standby and In-Match.
- **Wall-clock time**: SNTP-synced, fixed `Europe/London` POSIX TZ rule
  (`GMT0BST,M3.5.0/1,M10.5.0` — the OS's own `tzset()`/`localtime_r()`
  already handle the BST/GMT DST transition dates correctly; a full
  IANA timezone database is unnecessary for a personal,
  single-timezone device). `time()`/`localtime_r()`/`tzset()` come from
  `esp_idf_svc::sys` rather than the `libc` crate — `libc`'s
  xtensa-esp-idf target support doesn't expose `tzset()`, even though
  ESP-IDF's own newlib has it. Shows `"--:--"` before the first sync
  (or if it never manages to sync at all) rather than a wrong time.
- **`Application::refresh()`** (new, in `core`): re-renders the current
  state without applying a `Command` or touching storage. Needed
  because `handle()` only re-renders in response to a real state
  change, and none of `Command`'s variants are a safe no-op to send
  just to force a redraw (`Undo` would actually pop real history) —
  the clock needs the display to redraw periodically even when nothing
  else is happening, so `main()`'s idle loop now calls `refresh()`
  roughly once a minute instead of just sleeping forever.
- **SNTP never actually synced during testing** — no error, no
  success, just silence after initialization, on a WiFi network that
  looks like a mobile hotspot (`Three_99F1EB`). The likely cause is the
  network silently dropping outbound NTP (UDP/123), which mobile
  hotspots commonly do; the `"--:--"` fallback is doing exactly what it
  was designed to do in that case. Not investigated further — accepted
  as a real environmental limitation of the current test network, to
  revisit if it's still unsynced on whatever network the scoreboard
  actually lives on permanently.
- Also fixed in passing: 7 pre-existing `field_reassign_with_default`
  clippy findings in `core/tests/state_test.rs`, discovered because
  `cargo clippy` had apparently never been run directly inside `core`'s
  own directory before.

## Explicitly out of scope (this pass)

- LittleFS / persistence across reboots.
- The Controller and ESP-NOW link.
- The final HUB75-panel rendering layout (architecture.md's "Rendering"
  section) — this targets the small ST7789 bench screen only.
- WebSocket push, auth, input validation robustness at the REST boundary
  (slice 1's `copy_name`/`NAME_LEN` panic-risk discussion still applies —
  a malformed `Start-match` name could still panic the firmware; worth
  fixing before this is anything more than a personal test harness).
- Slice 3's full requirements (Controller hardware/design, error
  responses, anything beyond "the happy path works from a phone").
