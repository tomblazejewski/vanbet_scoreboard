# Slices

Work is broken into independently-gradable slices. Each gets its own
requirements doc, grilled individually (`/grilling`) before any
implementation starts — this file just tracks what the slices are and
their status.

## 1. Backend logic

Pure functions covering: incrementing points/Sets, serve computation, Set
progression (win-by-2/Deuce, history), undo/redo. Corresponds to
`lib/core` in [software-design.md](software-design.md) — no hardware, no
I/O, tested natively (ADR-0005).

**Status:** grilled, requirements resolved — see
[slices/01-backend-logic-requirements.md](slices/01-backend-logic-requirements.md).
`Unlock` cut from scope during implementation scoping (see that doc's
"Explicitly out of scope"). Not yet implemented.

## 2. Display protocol

Defines a Display port (protocol) and implements it against a small bench
display — a **1.14" TTGO** (ESP32 + integrated ST7789-class TFT) — rather
than the full 64x64 HUB75 panel, so the protocol can be exercised and what
renders can be tested as a function of state before the real panel is in
hand. This is the concrete instance of the "different display for
testing" requirement from the earlier architecture grilling (ADR-0004's
Display port).

**Status:** not yet defined.

## 3. Interacting with the microcontroller

REST API, the physical Controller's design and how it interacts with the
Display, and — importantly — a protocol/harness that lets interactions be
tested from a laptop instead of the physical Controller (mirrors slice 2's
"swap the real thing for a testable substitute" approach, applied to input
rather than output).

**Status:** not yet defined.

## Process

For each slice, before writing any code:
1. Draft a requirements doc for that slice alone.
2. Grill it (`/grill-with-docs`) until we reach shared understanding.
3. Only then implement.
