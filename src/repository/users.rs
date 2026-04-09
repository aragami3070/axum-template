use macroses::NewTypeDeref;
use std::ops::Deref;
use uuid::Uuid;

use crate::models::users::User;

#[derive(NewTypeDeref)]
pub struct Limit(pub u32);
#[derive(NewTypeDeref)]
pub struct Offset(pub u32);

pub trait UserRepository {
    async fn get_by_id(&self, id: &Uuid) -> Option<User>;
    async fn get(&self, offset: &Offset, limit: &Limit) -> Option<User>;
    async fn create(&self, user: User);
    async fn update(&self, user: User);
}

