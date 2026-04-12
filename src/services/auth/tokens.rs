use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::models::users::User;
use crate::repositories::tokens::{TokenRepo, TokenRepository};
use crate::repositories::users::{UserRepo, UserRepository};
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

    pub async fn refresh_tokens(&self, old_refresh_token: String) -> Result<Tokens> {
        let cur_claims = self.validate_refresh_token(&old_refresh_token).await?;

        // NOTE: если пользователь не найден, значит токен не валидный
        let user = match self.user_repo.get_by_id(&cur_claims.sub).await? {
            Some(u) => u,
            None => return Err(TokenError::InvalidToken),
        };

        self.generate_tokens(&user).await
    }

    pub async fn generate_tokens(&self, user: &User) -> Result<Tokens> {
        let access_token = self.generate_access_token(user)?;
        let refresh_token = self.generate_refresh_token(user)?;

        let _ = self
            .token_repo
            .create(
                self.token_repo.db_pool.clone().as_ref(),
                (&user.id, &refresh_token),
            )
            .await?;

        Ok(Tokens {
            access_token,
            refresh_token,
        })
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

    pub fn validate_access_token(&self, token: &str) -> Result<Claims> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);

        validation.leeway = 0;
        validation.required_spec_claims.insert("exp".to_string());

        let token_data = match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(t_d) => t_d,
            Err(e) if *e.kind() == jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                return Err(TokenError::Expired);
            }
            Err(e) => return Err(TokenError::Jwt(e)),
        };

        Ok(token_data.claims)
    }

    async fn validate_refresh_token(&self, token: &str) -> Result<Claims> {
        let decoding_key = DecodingKey::from_secret(self.secret_refresh.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);

        validation.leeway = 0;
        validation.required_spec_claims.insert("exp".to_string());

        let token_data = match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(t_d) => t_d,
            Err(e) if *e.kind() == jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                return Err(TokenError::Expired);
            }
            Err(e) => return Err(TokenError::Jwt(e)),
        };

        let is_valid_token = match self.token_repo.get(&token_data.claims.sub).await? {
            Some(u) => u == hash(token),
            None => return Err(TokenError::RefreshNotFound),
        };

        if is_valid_token {
            Ok(token_data.claims)
        } else {
            Err(TokenError::InvalidToken)
        }
    }
}
