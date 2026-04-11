use sqlx::{Executor, Pool, Postgres, postgres::PgRow};
use std::sync::Arc;
use uuid::Uuid;

pub trait TokenRepository {
    async fn get(&self, user_id: &Uuid) -> sqlx::Result<Option<String>>;
    async fn create<'e, E>(
        &self,
        executer: E,
        refreh_token_info: (&Uuid, &str),
    ) -> sqlx::Result<PgRow>
    where
        E: Executor<'e, Database = sqlx::Postgres>;
}

