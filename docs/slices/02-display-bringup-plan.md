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

**E. The REST layer** — `protocol.md`'s real endpoints via `esp-idf-svc`'s
HTTP server, each parsing JSON into a `Command` and calling
`Application::handle()`. This is what gets tested from a phone.

**F. Wire them together** — `Application`'s `Display` impl renders live
score/state to the screen as POST requests arrive.

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
