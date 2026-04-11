use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::models::users::User;
use crate::repository::tokens::{TokenRepo, TokenRepository};
use crate::repository::users::{UserRepo, UserRepository};
use crate::services::auth::hashing::hash;
use crate::{errors::tokens::TokenError, models::tokens::Tokens};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::{Database, Postgres};
use uuid::Uuid;

type Result<T> = std::result::Result<T, TokenError>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub role: String,
    pub exp: usize,
    pub jti: Option<String>,
}

#[derive(Clone)]
pub struct CurrentUser(pub Claims);

pub struct TokenService<Db: Database> {
    secret: Arc<String>,
    secret_refresh: Arc<String>,
    access_duration: usize,
    refresh_duration: usize,
    token_repo: Arc<TokenRepo<Db>>,
    user_repo: Arc<UserRepo<Postgres>>,
}

impl TokenService<Postgres> {
    /// Creates a new [`TokenService`].
    pub fn new(
        secret: Arc<String>,
        secret_refresh: Arc<String>,
        access_duration: usize,
        token_repo: Arc<TokenRepo<Postgres>>,
        user_repo: Arc<UserRepo<Postgres>>,
        refresh_duration: usize,
    ) -> Self {
        Self {
            secret,
            secret_refresh,
            token_repo,
            user_repo,
            access_duration,
            refresh_duration,
        }
    }

    fn generate_access_token(&self, user: &User) -> Result<String> {
        let exp = (SystemTime::now() + Duration::from_mins(self.access_duration as u64))
            .duration_since(UNIX_EPOCH)?
            .as_secs() as usize;

        let claims = Claims {
            sub: user.id,
            role: String::from(user.role.clone()),
            exp,
            jti: None,
        };

        Ok(encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?)
    }

    fn generate_refresh_token(&self, user: &User) -> Result<String> {
        let exp = (SystemTime::now() + Duration::from_mins(self.refresh_duration as u64))
            .duration_since(UNIX_EPOCH)?
            .as_secs() as usize;

        let claims = Claims {
            sub: user.id,
            role: String::from(user.role.clone()),
            exp,
            jti: Some(Uuid::new_v4().to_string()),
        };

        Ok(encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret_refresh.as_bytes()),
        )?)
    }
}
