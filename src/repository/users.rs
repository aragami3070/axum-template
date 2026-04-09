use macroses::NewTypeDeref;
use std::ops::Deref;

#[derive(NewTypeDeref)]
pub struct Limit(pub u32);
#[derive(NewTypeDeref)]
pub struct Offset(pub u32);

