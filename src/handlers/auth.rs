use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{post, put},
};
use utoipa::OpenApi;

use crate::{
    AppState,
    errors::{auth::AuthError, users::UserError},
    models::tokens::Tokens,
    repositories::is_unique_violation,
    schemas::{
        tokens::RefreshToken,
        users::{LoginUser, RegisterUser},
    },
    services::auth::hashing::hash,
};

pub struct AuthRouter;

impl AuthRouter {
    pub fn set_router() -> Router<AppState> {
        Router::new()
            .route("/registration", post(register))
            .route("/login", post(login))
            .route("/refresh_tokens", put(refresh_tokens))
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(register, login, refresh_tokens),
    components(schemas(LoginUser, RegisterUser, RefreshToken, Tokens))
)]
pub struct AuthDocs;

#[utoipa::path(
    post,
    path = "/registration",
    tag = "auth",
    request_body = RegisterUser,
    responses(
        (status = 200, description = "User registered successfully", body = Tokens),
        (status = 409, description = "User with this email already exists", body = String),
        (status = 500, description = "Internal database error", body = String)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(user_data): Json<RegisterUser>,
) -> Result<impl IntoResponse, AuthError> {
    let mut tx = state.begin_transaction().await?;

    let user = match state.user_repo.create(&mut *tx, user_data).await {
        Ok(user) => user,
        Err(e) if is_unique_violation(&e) => {
            return Err(AuthError::UserError(UserError::UserAlreadyExists));
        }
        Err(e) => return Err(AuthError::Db(e)),
    };

    let tokens = state.token_serv.generate_tokens(&mut tx, &user).await?;
    tx.commit().await?;

    Ok((StatusCode::OK, Json(tokens)))
}

#[utoipa::path(
    post,
    path = "/login",
    tag = "auth",
    request_body = LoginUser,
    responses(
        (status = 200, description = "Login successful", body = Tokens),
        (status = 401, description = "Invalid email or password", body = String),
        (status = 500, description = "Internal database error", body = String)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(user_data): Json<LoginUser>,
) -> Result<impl IntoResponse, AuthError> {
    let mut tx = state.begin_transaction().await?;

    let user = match state
        .user_repo
        .check_login(&mut *tx, &user_data.email, &hash(&user_data.password))
        .await?
    {
        Some(user) => user,
        None => return Err(AuthError::Unauthorized),
    };

    let tokens = state.token_serv.generate_tokens(&mut tx, &user).await?;
    tx.commit().await?;

    Ok((StatusCode::OK, Json(tokens)))
}

#[utoipa::path(
    put,
    path = "/refresh_tokens",
    tag = "auth",
    params (
        ("old_refresh_token" = RefreshToken, Query, description = "Old refresh token")
    ),
    responses(
        (status = 200, description = "Tokens refreshed successfully", body = Tokens),
        (status = 401, description = "Token is invalid or expired", body = String),
        (status = 500, description = "Internal database error", body = String)
    )
)]
pub async fn refresh_tokens(
    State(state): State<AppState>,
    Query(old_refresh_token): Query<RefreshToken>,
) -> Result<impl IntoResponse, AuthError> {
    let mut tx = state.begin_transaction().await?;

    let tokens = state
        .token_serv
        .refresh_tokens(&mut tx, old_refresh_token.token)
        .await?;
    tx.commit().await?;

    Ok((StatusCode::OK, Json(tokens)))
}
