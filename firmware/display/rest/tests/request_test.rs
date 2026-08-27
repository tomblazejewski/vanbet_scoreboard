//! Wire-format request bodies -> `Command`, including the REST-boundary
//! input validation `display-core` explicitly assumes has already
//! happened (see `docs/slices/01-backend-logic-requirements.md`).

use display_core::{Command, Side};
use display_rest::request::{PointRequest, RestError, ServerRequest, StartMatchRequest};

mod point_request {
    use super::*;

    #[test]
    fn valid_left_side_becomes_a_point_command() {
        let req: PointRequest = serde_json::from_str(r#"{"side":"left"}"#).unwrap();

        assert_eq!(req.into_command(), Ok(Command::Point { side: Side::Left }));
    }

    #[test]
    fn valid_right_side_becomes_a_point_command() {
        let req: PointRequest = serde_json::from_str(r#"{"side":"right"}"#).unwrap();

        assert_eq!(req.into_command(), Ok(Command::Point { side: Side::Right }));
    }

    #[test]
    fn invalid_side_string_is_rejected() {
        let req: PointRequest = serde_json::from_str(r#"{"side":"up"}"#).unwrap();

        assert_eq!(req.into_command(), Err(RestError::InvalidSide));
    }
}

mod server_request {
    use super::*;

    #[test]
    fn valid_side_becomes_a_set_server_command() {
        let req: ServerRequest = serde_json::from_str(r#"{"side":"right"}"#).unwrap();

        assert_eq!(
            req.into_command(),
            Ok(Command::SetServer { side: Side::Right })
        );
    }

    #[test]
    fn invalid_side_string_is_rejected() {
        let req: ServerRequest = serde_json::from_str(r#"{"side":""}"#).unwrap();

        assert_eq!(req.into_command(), Err(RestError::InvalidSide));
    }
}

mod start_match_request {
    use super::*;

    #[test]
    fn valid_body_becomes_a_start_match_command() {
        let req: StartMatchRequest =
            serde_json::from_str(r#"{"nameLeft":"ALEX","nameRight":"JORDAN","bestOf":5}"#).unwrap();

        assert_eq!(
            req.into_command(),
            Ok(Command::StartMatch {
                name_left: "ALEX".to_string(),
                name_right: "JORDAN".to_string(),
                best_of: 5,
            })
        );
    }

    #[test]
    fn name_left_exactly_at_the_byte_cap_is_accepted() {
        let name = "A".repeat(display_core::NAME_LEN);
        let req = StartMatchRequest {
            name_left: name.clone(),
            name_right: "JORDAN".to_string(),
            best_of: 5,
        };

        assert_eq!(
            req.into_command(),
            Ok(Command::StartMatch {
                name_left: name,
                name_right: "JORDAN".to_string(),
                best_of: 5
            })
        );
    }

    #[test]
    fn name_left_over_the_byte_cap_is_rejected() {
        let req = StartMatchRequest {
            name_left: "A".repeat(display_core::NAME_LEN + 1),
            name_right: "JORDAN".to_string(),
            best_of: 5,
        };

        assert_eq!(
            req.into_command(),
            Err(RestError::NameTooLong { field: "nameLeft" })
        );
    }

    #[test]
    fn name_right_over_the_byte_cap_is_rejected() {
        let req = StartMatchRequest {
            name_left: "ALEX".to_string(),
            name_right: "A".repeat(display_core::NAME_LEN + 1),
            best_of: 5,
        };

        assert_eq!(
            req.into_command(),
            Err(RestError::NameTooLong { field: "nameRight" })
        );
    }

    #[test]
    fn multi_byte_utf8_name_is_measured_in_bytes_not_chars() {
        // Each '字' is 3 UTF-8 bytes — 22 chars * 3 bytes = 66 > NAME_LEN (64),
        // so this must be rejected even though the char count looks small.
        let req = StartMatchRequest {
            name_left: "字".repeat(22),
            name_right: "JORDAN".to_string(),
            best_of: 5,
        };

        assert_eq!(
            req.into_command(),
            Err(RestError::NameTooLong { field: "nameLeft" })
        );
    }

    #[test]
    fn even_best_of_is_rejected() {
        let req = StartMatchRequest {
            name_left: "ALEX".to_string(),
            name_right: "JORDAN".to_string(),
            best_of: 4,
        };

        assert_eq!(req.into_command(), Err(RestError::InvalidBestOf));
    }

    #[test]
    fn best_of_above_the_cap_is_rejected() {
        let req = StartMatchRequest {
            name_left: "ALEX".to_string(),
            name_right: "JORDAN".to_string(),
            best_of: 13,
        };

        assert_eq!(req.into_command(), Err(RestError::InvalidBestOf));
    }

    #[test]
    fn best_of_zero_is_rejected() {
        let req = StartMatchRequest {
            name_left: "ALEX".to_string(),
            name_right: "JORDAN".to_string(),
            best_of: 0,
        };

        assert_eq!(req.into_command(), Err(RestError::InvalidBestOf));
    }

    #[test]
    fn best_of_at_the_cap_is_accepted() {
        let req = StartMatchRequest {
            name_left: "ALEX".to_string(),
            name_right: "JORDAN".to_string(),
            best_of: display_core::MAX_SETS,
        };

        assert_eq!(
            req.into_command(),
            Ok(Command::StartMatch {
                name_left: "ALEX".to_string(),
                name_right: "JORDAN".to_string(),
                best_of: display_core::MAX_SETS,
            })
        );
    }
}
