//! Pins `Application`'s orchestration: resume-and-render on construction,
//! and apply+save+render exactly once per `handle()` call. Uses hand-rolled
//! spies, not a mocking crate — `core` has zero external dependencies and
//! these are two-method interfaces. See `docs/software-design.md`'s
//! "The core is a pure function..." section.

use std::cell::RefCell;
use std::rc::Rc;

use display_core::{Application, Command, Display, MatchState, Storage};

#[derive(Clone)]
struct SpyDisplay {
    rendered: Rc<RefCell<Vec<MatchState>>>,
}

impl SpyDisplay {
    fn new() -> Self {
        Self {
            rendered: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl Display for SpyDisplay {
    fn render(&mut self, state: &MatchState) {
        self.rendered.borrow_mut().push(state.clone());
    }
}

struct FakeStorage {
    loaded: Option<MatchState>,
    saved: Rc<RefCell<Vec<MatchState>>>,
}

impl FakeStorage {
    fn empty() -> Self {
        Self {
            loaded: None,
            saved: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn with_saved_state(state: MatchState) -> Self {
        Self {
            loaded: Some(state),
            saved: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl Storage for FakeStorage {
    fn save(&mut self, state: &MatchState) {
        self.saved.borrow_mut().push(state.clone());
    }

    fn load(&self) -> Option<MatchState> {
        self.loaded.clone()
    }
}

#[test]
fn new_starts_at_standby_and_renders_immediately_when_storage_is_empty() {
    let display = SpyDisplay::new();
    let rendered = display.rendered.clone();

    let app = Application::new(display, FakeStorage::empty());

    assert_eq!(app.state(), &MatchState::default());
    assert_eq!(rendered.borrow().as_slice(), &[MatchState::default()]);
}

#[test]
fn new_resumes_from_storage_and_renders_the_loaded_state() {
    let saved_state = MatchState {
        active: true,
        score_left: 3,
        ..MatchState::default()
    };
    let display = SpyDisplay::new();
    let rendered = display.rendered.clone();

    let app = Application::new(display, FakeStorage::with_saved_state(saved_state.clone()));

    assert_eq!(app.state(), &saved_state);
    assert_eq!(rendered.borrow().as_slice(), &[saved_state]);
}

#[test]
fn handle_applies_saves_and_renders_exactly_once() {
    let display = SpyDisplay::new();
    let rendered = display.rendered.clone();
    let storage = FakeStorage::empty();
    let saved = storage.saved.clone();
    let mut app = Application::new(display, storage);

    app.handle(&Command::StartMatch {
        name_left: "Alice".to_string(),
        name_right: "Bob".to_string(),
        best_of: 5,
    });

    assert!(app.state().active, "the match started");
    assert_eq!(saved.borrow().len(), 1, "handle() saves exactly once");
    assert_eq!(
        rendered.borrow().len(),
        2,
        "one render on construction, one from handle()"
    );
    assert_eq!(saved.borrow()[0], *app.state());
    assert_eq!(rendered.borrow()[1], *app.state());
}

#[test]
fn refresh_renders_current_state_without_saving_or_changing_it() {
    let display = SpyDisplay::new();
    let rendered = display.rendered.clone();
    let storage = FakeStorage::empty();
    let saved = storage.saved.clone();
    let mut app = Application::new(display, storage);
    let state_before = app.state().clone();

    app.refresh();

    assert_eq!(app.state(), &state_before, "refresh() doesn't change state");
    assert_eq!(saved.borrow().len(), 0, "refresh() doesn't touch storage");
    assert_eq!(
        rendered.borrow().len(),
        2,
        "one render on construction, one from refresh()"
    );
    assert_eq!(rendered.borrow()[1], *app.state());
}
