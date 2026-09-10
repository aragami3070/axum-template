use std::sync::Arc;

use axum::{extract::Request, middleware::Next, response::Response};

use crate::{
    errors::{auth::AuthError, tokens::TokenError},
    services::auth::tokens::TokenService,
};

pub async fn auth_middleware(
    mut req: Request,
    next: Next,
    token_serv: Arc<TokenService>,
) -> Result<Response, AuthError> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(AuthError::Unauthorized)?;

    let (schema, token) = auth_header
        .split_once(' ')
        .ok_or(AuthError::TokenError(TokenError::InvalidAuthHeader))?;

    if !schema.eq_ignore_ascii_case("Bearer") {
        return Err(AuthError::TokenError(TokenError::InvalidAuthSchema));
    }

    let claims = match token_serv.validate_access_token(token) {
        Ok(cl) => cl,
        Err(er) => return Err(AuthError::TokenError(er)),
    };

    // NOTE: Если токен валидный, то извлеченные claims добавили в запрос
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
