# Working in this repo

Conventions that apply to every session working on this project.

## Slicing conventions

- **Slice requirement docs (`docs/slices/*.md`) never lock in an
  implementation language, file layout, or specific library/tooling.**
  They describe behavior, scope, and test scenarios — a slice's
  requirements should read the same whether it ends up built in C++, Rust,
  or anything else. Language and tooling choices are separate, revisable
  decisions recorded via ADR (see `docs/adr/`) and reflected in
  `software-design.md`, precisely so that changing the implementation
  language never requires re-grilling a slice. `docs/slices.md` (the
  tracking index, not a requirements doc) may report what was actually
  decided/built as a status fact — that's not the same as the requirements
  doc depending on it.

## Testing conventions

- **Assert on the whole object, not a chain of individual field asserts.**
  A test comparing resulting state makes one assertion against a
  fully-specified expected object (e.g. an expected `MatchState`), not a
  string of separate assertions on individual fields. If a comparison
  needs an equality operator that doesn't exist yet, add it rather than
  falling back to field-by-field assertions.
- **One test, one case.** Each test exercises exactly one scenario — don't
  fold multiple cases into a single test with several assert blocks. See
  `docs/slices/01-backend-logic-requirements.md`'s numbered test
  scenarios for the granularity expected: each numbered scenario is (at
  least) one test.
- **Group tests by concern, with informative titles.** Organize test
  files/suites around what's being tested (e.g. scoring, serve rotation,
  undo, Set/Match progression), not around implementation mechanics — and
  name each test for the behavior it proves, not a generic label.

## Code organization

- **Split source files by concern**, not one large file per module. The
  same concerns the tests are grouped by (scoring, serve rotation, undo,
  Set/Match progression, ...) should be visible as separate source
  modules, not folded into a single monolithic file.
- **Test file structure mirrors source file structure**, one-to-one. If
  source splits into a `scoring` module, a `serve` module, etc., tests
  split the same way — one test module per source module, not one big
  test file covering everything, and not a test layout that drifts from
  how the source itself is organized.
