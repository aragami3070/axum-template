use macroses::NewTypeDeref;
use sqlx::{Executor, Pool, Postgres};
use std::{ops::Deref, sync::Arc};
use uuid::Uuid;

use crate::{models::users::User, schemas::users::RegisterUser};

#[derive(NewTypeDeref)]
pub struct Limit(pub u64);
#[derive(NewTypeDeref)]
pub struct Offset(pub u64);

pub trait UserRepository {
    async fn get(&self, offset: &Offset, limit: &Limit) -> sqlx::Result<Vec<User>>;
    async fn get_by_id(&self, id: &Uuid) -> sqlx::Result<Option<User>>;
    async fn get_by_email(&self, email: &str) -> sqlx::Result<Option<User>>;
    async fn create_admin<'e, E>(&self, executer: E, user: RegisterUser) -> sqlx::Result<()>
    where
        E: Executor<'e, Database = sqlx::Postgres>;
    async fn create<'e, E>(&self, executer: E, user: RegisterUser) -> sqlx::Result<()>
    where
        E: Executor<'e, Database = sqlx::Postgres>;
    async fn update<'e, E>(&self, executer: E, user: User) -> sqlx::Result<()>
    where
        E: Executor<'e, Database = sqlx::Postgres>;
}

#[derive(Clone)]
pub struct UserRepo<Db>
where
    Db: sqlx::Database,
{
    db_pool: Arc<Pool<Db>>,
}

impl<Db: sqlx::Database> UserRepo<Db> {
    pub fn new(db_pool: Arc<Pool<Db>>) -> Self {
        Self { db_pool }
    }
}

impl UserRepository for UserRepo<Postgres> {
    async fn get(&self, offset: &Offset, limit: &Limit) -> sqlx::Result<Vec<User>> {
        sqlx::query_as!(
            User,
            "SELECT id, name, email, role, password_hash FROM users LIMIT $1 OFFSET $2",
            *limit.deref() as i64,
            *offset.deref() as i64
        )
        .fetch_all(self.db_pool.as_ref())
        .await
    }

    async fn get_by_id(&self, id: &Uuid) -> sqlx::Result<Option<User>> {
        sqlx::query_as!(
            User,
            "SELECT  id, name, email, role, password_hash FROM users WHERE id = $1",
            id
        )
        .fetch_optional(self.db_pool.as_ref())
        .await
    }

    async fn get_by_email(&self, email: &str) -> sqlx::Result<Option<User>> {
        sqlx::query_as!(
            User,
            "SELECT  id, name, email, role, password_hash FROM users WHERE email = $1",
            email
        )
        .fetch_optional(self.db_pool.as_ref())
        .await
    }

    async fn create_admin<'e, E>(&self, executer: E, user: RegisterUser) -> sqlx::Result<()>
    where
        E: Executor<'e, Database = sqlx::Postgres>,
    {
        todo!()
    }

    async fn create<'e, E>(&self, executer: E, user: RegisterUser) -> sqlx::Result<()>
    where
        E: Executor<'e, Database = sqlx::Postgres>,
    {
        todo!()
    }

    async fn update<'e, E>(&self, executer: E, user: User) -> sqlx::Result<()>
    where
        E: Executor<'e, Database = sqlx::Postgres>,
    {
        todo!()
    }
}
