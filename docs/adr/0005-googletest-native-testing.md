# Core logic is tested with GoogleTest under PlatformIO's `native` environment

Each firmware project gets an `env:native` PlatformIO environment (`platform =
native`, `test_framework = googletest`) that compiles and runs `test/` against
`lib/core` on the host machine — no ESP32 toolchain, no flashing, no simulator.
By default PlatformIO's test runner excludes `src/` from test builds, which lines
up exactly with ADR-0004's split: tests can only see the hardware-free core, so
there's no accidental path for a test to depend on an Arduino adapter.

**Considered and rejected:** doctest (lighter-weight, simpler syntax) — genuinely
a reasonable choice for this size of project, but GoogleTest was preferred,
notably for gmock's ease of building fake/mock port implementations
(`MockDisplay`, `MockStorage`) directly from the abstract port interfaces.
