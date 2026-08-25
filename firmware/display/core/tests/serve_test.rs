//! Pins `serve::compute_server`'s rotation math: every 2 points in normal
//! play, every 1 point once both sides reach 10 (Deuce) — see
//! `docs/match-rules.md` and test scenarios 2/3 in
//! `docs/slices/01-backend-logic-requirements.md`.

use display_core::{Side, compute_server};

mod compute_server_tests {
    use super::*;

    #[test]
    fn at_match_start_first_server_this_set_serves() {
        assert_eq!(compute_server(Side::Left, 0, 0), Side::Left);
    }

    #[test]
    fn first_server_still_serves_the_second_point_of_their_turn() {
        assert_eq!(compute_server(Side::Left, 1, 0), Side::Left);
    }

    #[test]
    fn server_switches_after_two_points_in_normal_play() {
        assert_eq!(compute_server(Side::Left, 1, 1), Side::Right);
    }

    #[test]
    fn server_switches_back_after_four_points_in_normal_play() {
        assert_eq!(compute_server(Side::Left, 2, 2), Side::Left);
    }

    #[test]
    fn server_at_deuce_onset_continues_from_normal_rotation() {
        // 10-10 is reached after 20 points — the same point count that
        // would put the normal every-2 rotation back on first_server_this_set,
        // so there's no discontinuity right at the Deuce boundary.
        assert_eq!(compute_server(Side::Left, 10, 10), Side::Left);
    }

    #[test]
    fn server_switches_every_single_point_once_in_deuce() {
        // Only one point past the 10-10 boundary — under the normal every-2
        // rule this wouldn't have switched yet, so this pins the Deuce
        // exception specifically.
        assert_eq!(compute_server(Side::Left, 11, 10), Side::Right);
    }
}
