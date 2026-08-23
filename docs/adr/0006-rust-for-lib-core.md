# `lib/core`'s state-manipulation logic is Rust, not C++

The Display's pure state-manipulation core — `apply()` and the modules
behind it (scoring, serve rotation, undo, Set/Match progression) — is
implemented in Rust, tested with `cargo test` on the host machine, not C++
with GoogleTest under PlatformIO's `env:native`. This amends
[ADR-0004](0004-hexagonal-core-with-display-command-storage-ports.md): the
hexagonal shape it describes (pure functions behind `Display`/`Storage`/
`Command` ports) stands unchanged — only the language for this one
component changes. It supersedes [ADR-0005](0005-googletest-native-testing.md)
for this component specifically; ADR-0005 still applies to any part of
either firmware that ends up staying in C++.

Neither the Display's hardware-facing adapters (HUB75 rendering, WiFi/web
server, LittleFS) nor the Controller firmware are decided by this ADR —
those are separate, later choices. In particular, whether the eventual
adapter layer is Rust too (no FFI needed) or stays C++ (Rust core called
via FFI) doesn't need answering now: no adapter code exists yet — slice 1
explicitly excludes it — so there's nothing to be consistent with today.

Slice requirement docs (`docs/slices/*.md`) describe behavior and scope
only, never a specific implementation language or file layout — see
`CLAUDE.md`. That's what makes a decision like this one possible without
re-grilling a slice's requirements: this ADR is where the language
commitment lives, not the requirements doc.

**Considered and rejected:** keeping `lib/core` in C++ for consistency with
the not-yet-built hardware adapters, pre-empting a future Rust↔C++ FFI
boundary — rejected because that boundary may never need to exist (the
adapters could just as easily end up in Rust too), and pre-deciding it now
would mean guessing at a slice that hasn't been grilled yet.

**Consequence:** `firmware/display`'s slice-1 deliverables move from a
PlatformIO/C++ project to a Cargo/Rust crate. The Checkpoint 1 work already
on the `slice-1-scoping` branch (C++ `command.h` + GoogleTest) is
superseded by a Rust equivalent on the same branch, not carried forward.
