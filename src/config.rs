use sqlx::Pool;
use sqlx::Postgres;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;

use crate::repository::users::UserRepo;

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

pub async fn get_db_pool(database_url: &str) -> sqlx::PgPool {
    PgPoolOptions::new().connect(database_url).await.unwrap()
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<Pool<Postgres>>,
    pub user_repo: Arc<UserRepo<Postgres>>,
    pub secret_key: String,
    pub secret_refresh_key: String,
}

impl AppState {
    pub fn new(
        db_pool: Arc<Pool<Postgres>>,
        secret_key: String,
        secret_refresh_key: String,
    ) -> Self {
        Self {
            user_repo: Arc::new(UserRepo::new(db_pool.clone())),
            db_pool,
            secret_key,
            secret_refresh_key,
        }
    }
}
