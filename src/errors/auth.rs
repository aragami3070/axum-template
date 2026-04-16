use axum::response::IntoResponse;
use http::StatusCode;
use thiserror::Error;

use crate::errors::{tokens, users};

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("Token service error: {0}")]
    TokenError(#[from] tokens::TokenError),

    #[error("Token service error: {0}")]
    UserError(#[from] users::UserError),

    #[error("User not found")]
    Unauthorized,

    #[error("User doesn't have sufficient rights for this action.")]
    Forbidden,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Db(er) => (StatusCode::INTERNAL_SERVER_ERROR, er.to_string()).into_response(),
            Self::TokenError(er) => er.into_response(),
            Self::UserError(er) => er.into_response(),
            Self::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "User unauthorized".to_string()).into_response()
            }

            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "User doesn't have sufficient rights for this action.".to_string(),
            )
                .into_response(),
        }
    }
}
