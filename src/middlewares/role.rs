use axum::{extract::Request, middleware::Next, response::Response};

use crate::{errors::auth::AuthError, models::users::Role, services::auth::tokens::Claims};

pub async fn role_middleware(
    req: Request,
    next: Next,
    allowed_roles: Vec<Role>,
) -> Result<Response, AuthError> {
    let claims = req.extensions().get::<Claims>().cloned();
    let claims = match claims {
        Some(cl) => cl,
        None => return Err(AuthError::Unauthorized),
    };

    if !allowed_roles.contains(&Role::from(claims.role)) {
        return Err(AuthError::Forbidden);
    }
    Ok(next.run(req).await)
}
