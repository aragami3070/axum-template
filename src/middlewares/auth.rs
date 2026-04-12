use std::{
    sync::Arc,
    task::{Context, Poll},
};

use http::Request;
use sqlx::Postgres;
use tower::{Layer, Service};

use crate::{errors::auth::AuthError, services::auth::tokens::TokenService};

#[derive(Clone)]
pub struct AuthLayer {
    pub token_serv: Arc<TokenService<Postgres>>,
}
