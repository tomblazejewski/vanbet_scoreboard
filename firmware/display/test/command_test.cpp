#include <gtest/gtest.h>

#include "command.h"

// command.h's operator== is what the whole-object-assert testing convention
// (CLAUDE.md) depends on for every other test file — pin its behavior down
// directly, including the "only the live prefix matters" rule for the
// history/undoStack arrays (see docs/slices/01-backend-logic-requirements.md).

TEST(MatchStateEquality, DefaultConstructedStatesAreEqual) {
  MatchState a;
  MatchState b;

  EXPECT_EQ(a, b);
}

TEST(MatchStateEquality, DifferingScoreIsNotEqual) {
  MatchState a;
  MatchState b;
  b.scoreLeft = 3;

  EXPECT_NE(a, b);
}

TEST(MatchStateEquality, DifferingLiveHistoryEntryIsNotEqual) {
  MatchState a;
  a.historyCount = 1;
  a.history[0] = SetResult{11, 9};

  MatchState b;
  b.historyCount = 1;
  b.history[0] = SetResult{11, 7};

  EXPECT_NE(a, b);
}

TEST(MatchStateEquality, DiffersOnlyPastHistoryCountAreStillEqual) {
  MatchState a;
  a.historyCount = 0;
  a.history[3] = SetResult{11, 9};  // stale buffer contents past the count

  MatchState b;
  b.historyCount = 0;
  b.history[3] = SetResult{5, 2};   // different stale contents, same prefix

  EXPECT_EQ(a, b);
}

TEST(MatchStateEquality, DiffersOnlyPastUndoCountAreStillEqual) {
  MatchState a;
  a.undoCount = 0;
  a.undoStack[10].scoreLeft = 7;

  MatchState b;
  b.undoCount = 0;
  b.undoStack[10].scoreLeft = 2;

  EXPECT_EQ(a, b);
}
