# Working in this repo

Conventions that apply to every session working on this project.

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
  undo, Set/Match progression, Unlock), not around implementation
  mechanics — and name each test for the behavior it proves, not a
  generic label.
