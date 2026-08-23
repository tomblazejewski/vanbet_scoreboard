# Match/Controller logic is a hardware-free core behind Display, Command, and Storage ports

Both firmware projects split into a pure C++ `lib/core` (no Arduino/ESP-IDF/hardware
headers at all) and a thin `src/` adapter layer. The core exposes three abstract
ports — **Display** (`render(state)`), **Command** (a tagged `Command` value type
flowing through a single-writer queue, so concurrent ESP-NOW/WebSocket callbacks
never mutate state directly), and **Storage** (`save`/`load`) — and state transitions
are pure functions (`apply(state, command) -> state`), with a thin `Application`
shell responsible for the one non-pure job: calling `storage.save()` and
`display.render()` after each transition. Adapters (the real HUB75 driver, ESP-NOW,
LittleFS, the phone's WebSocket/HTTP handlers) implement the ports and live only in
`src/`; `test/` links against `lib/core` alone via GoogleTest (ADR-0005) and never
touches Arduino headers even at compile time.

**Considered and rejected:** the first attempt at this firmware (scrapped) wrote
match logic, ESP-NOW handling, the web server, and panel rendering all inline in one
`main.cpp` — impossible to unit test without real hardware, and impossible to swap
the display or input source for bench testing.

**Consequence:** every new hardware capability needs a port method added
deliberately (not just an Arduino call dropped into whatever function needed it),
and the exact Command-queue mechanism reconciling concurrent input sources is
intentionally left open past this ADR — it's a real design question on its own,
not resolved by "there's a queue."
