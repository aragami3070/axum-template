use enum_iterator::{Sequence, all};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::users::UserError, schemas::users::RegisterUser, services::auth::hashing::hash,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: Role,
    #[serde(skip_serializing)]
    pub password_hash: String,
}

impl User {
    pub fn from_register(value: RegisterUser, secret: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: value.name,
            email: value.email,
            role: Role::User,
            password_hash: hash(&value.password, secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Sequence, PartialEq)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum Role {
    User,
    Admin,
}

impl Role {
    pub fn all() -> Vec<Self> {
        all::<Role>().collect()
    }

    pub fn admin_only() -> Vec<Self> {
        vec![Role::Admin]
    }
}

impl TryFrom<String> for Role {
    type Error = UserError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "user" => Ok(Role::User),
            "admin" => Ok(Role::Admin),
            _ => Err(UserError::InvalidRole),
        }
    }
}

impl From<Role> for String {
    fn from(value: Role) -> Self {
        match value {
            Role::User => String::from("user"),
            Role::Admin => String::from("admin"),
        }
    }
}
