use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{schemas::users::RegisterUser, services::auth::password_hashing::hash_password};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: Role,
    #[serde(skip_serializing)]
    pub password_hash: String,
}

impl From<RegisterUser> for User {
    fn from(value: RegisterUser) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: value.name,
            email: value.email,
            role: Role::User,
            password_hash: hash_password(&value.password),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum Role {
    User,
    Admin,
}

impl From<String> for Role {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "user" => Role::User,
            "admin" => Role::Admin,
            _ => panic!("Role must be admin or user"),
        }
    }
}
