use axum::response::IntoResponse;
use http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("User not found")]
    NotFound,
}

impl IntoResponse for UserError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Db(er) => (StatusCode::INTERNAL_SERVER_ERROR, er.to_string()).into_response(),
            Self::NotFound => (StatusCode::NOT_FOUND, "User not found".to_string()).into_response(),
        }
    }
}
