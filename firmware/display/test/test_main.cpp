// GoogleTest entry point for env:native. The googletest package PlatformIO
// pulls in here doesn't bundle gtest_main, so this is the one file in test/
// that isn't a mirror of a lib/core source file — just the runner.
#include <gtest/gtest.h>

int main(int argc, char** argv) {
  ::testing::InitGoogleTest(&argc, argv);
  return RUN_ALL_TESTS();
}
