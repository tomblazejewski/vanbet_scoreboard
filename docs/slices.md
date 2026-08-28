# Slices

Work is broken into independently-gradable slices. Each gets its own
requirements doc, grilled individually (`/grilling`) before any
implementation starts — this file just tracks what the slices are and
their status.

## 1. Backend logic

Pure functions covering: incrementing points/Sets, serve computation, Set
progression (win-by-2/Deuce, history), undo/redo. Corresponds to the
Display's state-manipulation core in [software-design.md](software-design.md)
— no hardware, no I/O, tested on the host machine.

**Status:** implemented. See
[slices/01-backend-logic-requirements.md](slices/01-backend-logic-requirements.md)
for the resolved requirements (`Unlock` cut from scope — see that doc's
"Explicitly out of scope"). Implementation language is Rust — see
[ADR-0006](adr/0006-rust-for-lib-core.md) (a status fact, not part of the
requirements doc itself — see `CLAUDE.md`'s "Slicing conventions"). All 8
checkpoints landed (`firmware/display/core`); `ports.rs`/`application.rs`
(wiring the core to real/fake hardware) are a later integration point,
untouched by this slice.

## 2. Display protocol

Defines a Display port (protocol) and implements it against a small bench
display — a **1.14" TTGO** (ESP32 + integrated ST7789-class TFT) — rather
than the full 64x64 HUB75 panel, so the protocol can be exercised and what
renders can be tested as a function of state before the real panel is in
hand. This is the concrete instance of the "different display for
testing" requirement from the earlier architecture grilling (ADR-0004's
Display port).

**Status:** bring-up plan complete (all checkpoints A–F) — brought up
together with slice 3's REST layer against real hardware (a LILYGO TTGO
T-Display, exactly the bench display described above). See
[slices/02-display-bringup-plan.md](slices/02-display-bringup-plan.md) for
the checkpoint plan and what was found along the way; this was
lighter-weight than a full behavior grilling (see that doc's intro), and
the final layout from architecture.md's "Rendering" section (beyond plain
score text) is still its own future pass.

## 3. Interacting with the microcontroller

REST API, the physical Controller's design and how it interacts with the
Display, and — importantly — a protocol/harness that lets interactions be
tested from a laptop instead of the physical Controller (mirrors slice 2's
"swap the real thing for a testable substitute" approach, applied to input
rather than output).

**Status:** partially in progress — the phone/HTTP half of
[protocol.md](protocol.md) is done and confirmed against real hardware
(see [slices/02-display-bringup-plan.md](slices/02-display-bringup-plan.md)'s
Checkpoint E), built alongside slice 2 since it's needed to exercise the
Display port. Not yet grilled as its own slice: the Controller's design
(including whether it's even dedicated ESP32 hardware — an open question
raised mid-session), error-response conventions beyond the happy-path
`400`/`409` this pass needed, and any auth-adjacent questions.

## Process

For each slice, before writing any code:
1. Draft a requirements doc for that slice alone.
2. Grill it (`/grill-with-docs`) until we reach shared understanding.
3. Only then implement.

Implementation language and tooling are a separate decision from a
slice's requirements — recorded via ADR when they carry real trade-offs,
not baked into the requirements doc. See `CLAUDE.md`'s "Slicing
conventions."
