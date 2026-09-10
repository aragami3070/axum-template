use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Postgres, Transaction};
use std::env;
use std::sync::Arc;

use crate::repositories::tokens::TokenRepo;
use crate::repositories::users::UserRepo;
use crate::services::auth::tokens::TokenService;

pub struct Config {
    pub database_url: String,
    pub secret_key: String,
    pub secret_refresh_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect(".env not loaded"),
            secret_key: env::var("JWT_SECRET").expect(".env not loaded"),
            secret_refresh_key: env::var("JWT_SECRET_REFRESH").expect(".env not loaded"),
        }
    }
}

pub async fn get_db_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new().connect(database_url).await.unwrap()
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,

    // NOTE: Сервисы
    pub token_serv: Arc<TokenService>,
}

impl AppState {
    pub fn new(db_pool: PgPool, secret_key: String, secret_refresh_key: String) -> Self {
        let secret_key = Arc::new(secret_key);
        let secret_refresh_key = Arc::new(secret_refresh_key);

        // NOTE: Репозитории
        let user_repo = Arc::new(UserRepo);
        let token_repo = Arc::new(TokenRepo);

        // NOTE: Сервисы
        let token_serv = Arc::new(TokenService::new(
            secret_key.clone(),
            secret_refresh_key.clone(),
            15,
            token_repo.clone(),
            user_repo.clone(),
            1440,
        ));

        Self {
            db_pool,
            token_serv,
        }
    }

    pub async fn begin_transaction(&self) -> sqlx::Result<Transaction<'static, Postgres>> {
        self.db_pool.begin().await
    }
}
