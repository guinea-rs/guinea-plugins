//! What a route says when it cannot answer.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// A status and a sentence, which is the whole of the API's error model.
pub struct Failure(pub StatusCode, pub String);

impl Failure {
    pub fn not_found(what: impl std::fmt::Display) -> Self {
        Failure(StatusCode::NOT_FOUND, what.to_string())
    }

    pub fn bad_request(what: impl std::fmt::Display) -> Self {
        Failure(StatusCode::BAD_REQUEST, what.to_string())
    }

    /// The application cannot do what was asked of it right now.
    pub fn refused(what: impl std::fmt::Display) -> Self {
        Failure(StatusCode::CONFLICT, what.to_string())
    }

    /// The application was asked and has not answered.
    pub fn timed_out(what: impl std::fmt::Display) -> Self {
        Failure(StatusCode::GATEWAY_TIMEOUT, what.to_string())
    }
}

impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        let Failure(status, message) = self;

        (status, axum::Json(serde_json::json!({ "error": message }))).into_response()
    }
}
