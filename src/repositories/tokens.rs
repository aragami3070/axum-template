use sqlx::{Executor, Postgres, postgres::PgQueryResult};
use uuid::Uuid;

use crate::services::auth::hashing::hash;

pub trait TokenRepository {
    async fn get<'e, E>(&self, executor: E, user_id: &Uuid) -> sqlx::Result<Option<String>>
    where
        E: Executor<'e, Database = Postgres>;
    async fn create<'e, E>(
        &self,
        executor: E,
        refresh_token_info: (&Uuid, &str),
    ) -> sqlx::Result<PgQueryResult>
    where
        E: Executor<'e, Database = Postgres>;
}

#[derive(Clone, Default)]
pub struct TokenRepo;

impl TokenRepository for TokenRepo {
    async fn get<'e, E>(&self, executor: E, user_id: &Uuid) -> sqlx::Result<Option<String>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_scalar!(
            "SELECT token FROM refresh_tokens
            WHERE user_id = $1",
            user_id
        )
        .fetch_optional(executor)
        .await
    }

    async fn create<'e, E>(
        &self,
        executor: E,
        refresh_token_info: (&Uuid, &str),
    ) -> sqlx::Result<PgQueryResult>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query!(
            "INSERT INTO refresh_tokens (user_id, token) VALUES ($1, $2)
            ON CONFLICT (user_id) DO UPDATE SET token = $2",
            refresh_token_info.0,
            hash(refresh_token_info.1)
        )
        .execute(executor)
        .await
    }
}
