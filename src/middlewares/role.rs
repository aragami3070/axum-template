use std::task::{Context, Poll};

use axum::response::IntoResponse;
use http::Request;
use tower::{Layer, Service};

use crate::{errors::auth::AuthError, models::users::Role, services::auth::tokens::Claims};

#[derive(Clone)]
pub struct RoleLayer {
    allowed_roles: Vec<Role>,
}

#[derive(Clone)]
pub struct RoleGuardMiddleware<S> {
    inner: S,
    allowed_roles: Vec<Role>,
}
