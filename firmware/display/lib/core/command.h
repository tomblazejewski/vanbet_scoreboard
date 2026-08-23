#pragma once

// Data shapes and constants for the Display's match logic core. See
// docs/architecture.md's "Data model" and docs/slices/01-backend-logic-requirements.md.

#include <cstdint>
#include <cstring>
#include <string>

// Fixed rules — see docs/match-rules.md. Not configurable.
constexpr uint8_t POINTS_TO_WIN = 11;

// Sized to the capped maximum bestOf (odd, <= 11) — see
// docs/slices/01-backend-logic-requirements.md.
constexpr uint8_t MAX_SETS = 11;

// Generous headroom for a full Match's worth of points. Once full, pushing a
// new snapshot evicts the oldest — the ceiling behaves as "oldest undo
// capability quietly stops being available," not an error. See undo.h.
constexpr uint16_t MAX_UNDO = 200;

// Buffer size for a Player name. Length itself is a REST-boundary
// validation concern (slice 3) — this just needs a buffer big enough for
// whatever the REST layer already validated.
constexpr uint8_t NAME_LEN = 16;

// Left or Right — fixed for the whole Match by which Controller button is
// pressed, not by player identity. See CONTEXT.md.
enum class Side : uint8_t { LEFT = 0, RIGHT = 1 };

// A completed Set's final score, appended to MatchState::history.
struct SetResult {
  uint8_t scoreLeft = 0;
  uint8_t scoreRight = 0;
};

inline bool operator==(const SetResult& a, const SetResult& b) {
  return a.scoreLeft == b.scoreLeft && a.scoreRight == b.scoreRight;
}

// Pushed before every point is applied; popped + restored on Undo.
// Deliberately NOT used for Set-server corrections — undo is for points,
// not serve corrections, which have their own dedicated mechanism.
struct UndoSnapshot {
  uint8_t scoreLeft = 0;
  uint8_t scoreRight = 0;
  uint8_t setsWonLeft = 0;
  uint8_t setsWonRight = 0;
  Side server = Side::LEFT;
  Side firstServerThisSet = Side::LEFT;
  uint8_t historyCount = 0;
};

inline bool operator==(const UndoSnapshot& a, const UndoSnapshot& b) {
  return a.scoreLeft == b.scoreLeft && a.scoreRight == b.scoreRight &&
         a.setsWonLeft == b.setsWonLeft && a.setsWonRight == b.setsWonRight &&
         a.server == b.server &&
         a.firstServerThisSet == b.firstServerThisSet &&
         a.historyCount == b.historyCount;
}

// The Display's sole authoritative state. active == false is Standby;
// active == true is In-Match (whether or not the Match is decided — see
// docs/architecture.md's "Match lifecycle").
struct MatchState {
  bool active = false;
  char nameLeft[NAME_LEN] = {};
  char nameRight[NAME_LEN] = {};
  uint8_t bestOf = 0;
  uint8_t setsWonLeft = 0;
  uint8_t setsWonRight = 0;
  uint8_t scoreLeft = 0;   // current Set in progress
  uint8_t scoreRight = 0;
  Side server = Side::LEFT;
  Side firstServerThisSet = Side::LEFT;  // anchor for computing server
  SetResult history[MAX_SETS] = {};
  uint8_t historyCount = 0;
  UndoSnapshot undoStack[MAX_UNDO] = {};
  uint16_t undoCount = 0;
};

// Compares scalar fields directly; for history/undoStack, only the live
// prefix ([0, historyCount) / [0, undoCount)) — entries past the count are
// stale buffer contents, not state.
inline bool operator==(const MatchState& a, const MatchState& b) {
  if (a.active != b.active) return false;
  if (std::strncmp(a.nameLeft, b.nameLeft, NAME_LEN) != 0) return false;
  if (std::strncmp(a.nameRight, b.nameRight, NAME_LEN) != 0) return false;
  if (a.bestOf != b.bestOf) return false;
  if (a.setsWonLeft != b.setsWonLeft) return false;
  if (a.setsWonRight != b.setsWonRight) return false;
  if (a.scoreLeft != b.scoreLeft) return false;
  if (a.scoreRight != b.scoreRight) return false;
  if (a.server != b.server) return false;
  if (a.firstServerThisSet != b.firstServerThisSet) return false;
  if (a.historyCount != b.historyCount) return false;
  if (a.undoCount != b.undoCount) return false;
  for (uint8_t i = 0; i < a.historyCount; ++i) {
    if (!(a.history[i] == b.history[i])) return false;
  }
  for (uint16_t i = 0; i < a.undoCount; ++i) {
    if (!(a.undoStack[i] == b.undoStack[i])) return false;
  }
  return true;
}

inline bool operator!=(const MatchState& a, const MatchState& b) {
  return !(a == b);
}

// A single tagged value type, not an interface with a method per action —
// composes with a queue directly (see docs/software-design.md's rationale).
// Unlock is cut from MVP scope — see
// docs/slices/01-backend-logic-requirements.md's "Explicitly out of scope."
enum class CommandType : uint8_t {
  START_MATCH,
  POINT,
  UNDO,
  SET_SERVER,
  CLOSE,
};

struct Command {
  CommandType type;
  Side side = Side::LEFT;            // POINT, SET_SERVER
  std::string nameLeft, nameRight;   // START_MATCH
  uint8_t bestOf = 0;                // START_MATCH
};
