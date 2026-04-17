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
    repositories::{is_unique_violation, users::UserRepository},
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
    let repo = state.user_repo.clone();
    let token_serv = state.token_serv.clone();

    match repo
        .create(repo.db_pool.clone().as_ref(), user_data.clone())
        .await
    {
        Ok(user) => Ok((
            StatusCode::OK,
            Json(token_serv.generate_tokens(&user).await?),
        )),
        Err(e) if is_unique_violation(&e) => {
            Err(AuthError::UserError(UserError::UserAlreadyExists))
        }
        Err(e) => Err(AuthError::Db(e)),
    }
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
    let repo = state.user_repo.clone();
    let token_serv = state.token_serv.clone();

    match repo
        .check_login(&user_data.email, &hash(&user_data.password))
        .await?
    {
        Some(user) => Ok((
            StatusCode::OK,
            Json(token_serv.generate_tokens(&user).await?),
        )),
        None => Err(AuthError::Unauthorized),
    }
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
    let token_serv = state.token_serv.clone();

    Ok((
        StatusCode::OK,
        Json(token_serv.refresh_tokens(old_refresh_token.token).await?),
    ))
}
