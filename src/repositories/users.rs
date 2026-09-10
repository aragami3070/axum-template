use macroses::NewTypeDeref;
use serde::Deserialize;
use sqlx::{Executor, Postgres, postgres::PgQueryResult};
use std::ops::Deref;
use uuid::Uuid;

use crate::{
    models::users::{Role, User},
    schemas::users::RegisterUser,
};

#[derive(NewTypeDeref, Deserialize)]
pub struct Limit(pub u64);
#[derive(NewTypeDeref, Deserialize)]
pub struct Offset(pub u64);

#[derive(Clone, Default)]
pub struct UserRepo;

impl UserRepo {
    pub async fn get<'e, E>(
        &self,
        executor: E,
        offset: &Offset,
        limit: &Limit,
    ) -> sqlx::Result<Vec<User>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as!(
            User,
            "SELECT id, name, email, role AS \"role: Role\", password_hash
            FROM users
            LIMIT $1 OFFSET $2",
            *limit.deref() as i64,
            *offset.deref() as i64
        )
        .fetch_all(executor)
        .await
    }

    pub async fn get_by_id<'e, E>(&self, executor: E, id: &Uuid) -> sqlx::Result<Option<User>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as!(
            User,
            "SELECT id, name, email, role AS \"role: Role\", password_hash
            FROM users
            WHERE id = $1",
            id
        )
        .fetch_optional(executor)
        .await
    }

    pub async fn get_by_email<'e, E>(&self, executor: E, email: &str) -> sqlx::Result<Option<User>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as!(
            User,
            "SELECT  id, name, email, role AS \"role: Role\", password_hash
            FROM users
            WHERE email = $1",
            email
        )
        .fetch_optional(executor)
        .await
    }

    pub async fn check_login<'e, E>(
        &self,
        executor: E,
        email: &str,
        password_hash: &str,
    ) -> sqlx::Result<Option<User>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as!(
            User,
            "SELECT  id, name, email, role AS \"role: Role\", password_hash
            FROM users
            WHERE email = $1 AND password_hash = $2",
            email,
            password_hash
        )
        .fetch_optional(executor)
        .await
    }

    pub async fn create_admin<'e, E>(&self, executor: E, user: RegisterUser) -> sqlx::Result<User>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let mut user_data: User = user.into();
        user_data.role = crate::models::users::Role::Admin;
        sqlx::query_as!(
            User,
            "INSERT INTO users (id, name, email, role, password_hash)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, email, role AS \"role: Role\", password_hash;",
            user_data.id,
            user_data.name,
            user_data.email,
            String::from(user_data.role),
            user_data.password_hash
        )
        .fetch_one(executor)
        .await
    }

    pub async fn create<'e, E>(&self, executor: E, user: RegisterUser) -> sqlx::Result<User>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let user_data: User = user.into();
        sqlx::query_as!(
            User,
            "INSERT INTO users (id, name, email, role, password_hash)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, email, role AS \"role: Role\", password_hash;",
            user_data.id,
            user_data.name,
            user_data.email,
            String::from(user_data.role),
            user_data.password_hash
        )
        .fetch_one(executor)
        .await
    }

    pub async fn update<'e, E>(&self, executor: E, user: User) -> sqlx::Result<PgQueryResult>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query!(
            "UPDATE users
            SET name = $2, email = $3, role = $4, password_hash = $5
            WHERE id = $1;",
            user.id,
            user.name,
            user.email,
            String::from(user.role),
            user.password_hash
        )
        .execute(executor)
        .await
    }
}
