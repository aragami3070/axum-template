use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware::from_fn,
    response::IntoResponse,
    routing::{get, post, put},
};
use utoipa::OpenApi;

use crate::{
    AppState,
    errors::users::UserError,
    middlewares::{auth::auth_middleware, role::role_middleware},
    models::users::{Role, User},
    repositories::{
        is_unique_violation,
        users::{Limit, Offset, UserRepository},
    },
    schemas::users::{RegisterUser, UserResponse},
    services::auth::tokens::Claims,
};

pub struct UserRouter;

impl UserRouter {
    pub fn set_router(state: AppState) -> Router<AppState> {
        Router::new()
            .route("/my_profile", get(my_profile))
            .route("/create_admin", post(create_admin))
            .route("/", put(update))
            .route("/{offset}/{page_limit}", get(get_with_pagination))
            .route_layer(from_fn(move |req, next| async move {
                role_middleware(req, next, Role::all()).await
            }))
            .route_layer({
                let token_serv = state.token_serv.clone();
                from_fn(move |req, next| {
                    let token_serv = token_serv.clone();
                    async move { auth_middleware(req, next, token_serv.clone()).await }
                })
            })
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(my_profile, create_admin, update, get_with_pagination),
    components(schemas(RegisterUser))
)]
pub struct UserDocs;

#[utoipa::path(
    get,
    tag = "user",
    // NOTE: обязательно надо добавить, чтобы с свагера на эту ручку отправлялся токен
    security(
        ("bearer_auth" = [])
    ),
    path = "/my_profile",
    responses(
        (status = 200, description = "User retrieved successfully", body = UserResponse),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal database error", body = String)
    )
)]
pub async fn my_profile(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, UserError> {
    let repo = state.user_repo.clone();
    // NOTE: пример получения id пользователя из claims
    match repo.get_by_id(&claims.sub).await? {
        Some(user) => Ok((StatusCode::OK, Json(UserResponse::from(user)))),
        None => Err(UserError::NotFound),
    }
}

#[utoipa::path(
    post,
    tag = "user",
    security(
        ("bearer_auth" = [])
    ),
    path = "/create_admin",
    request_body = RegisterUser,
    responses(
        (status = 200, description = "Admin created successfully", body = UserResponse),
        (status = 409, description = "User with this email already exists", body = String),
        (status = 500, description = "Internal database error", body = String)
    )
)]
pub async fn create_admin(
    State(state): State<AppState>,
    Json(user_data): Json<RegisterUser>,
) -> Result<impl IntoResponse, UserError> {
    let repo = state.user_repo.clone();

    match repo
        .create_admin(repo.db_pool.clone().as_ref(), user_data)
        .await
    {
        Ok(admin) => Ok((StatusCode::OK, Json(UserResponse::from(admin)))),
        Err(e) if is_unique_violation(&e) => Err(UserError::UserAlreadyExists),
        Err(e) => Err(UserError::Db(e)),
    }
}

#[utoipa::path(
    put,
    tag = "user",
    security(
        ("bearer_auth" = [])
    ),
    path = "",
    request_body = RegisterUser,
    responses(
        (status = 204, description = "Data updated"),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal database error", body = String)
    )
)]
/// NOTE: user меняет инфу о себе (кроме id и роли)
pub async fn update(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Json(user_data): Json<RegisterUser>,
) -> Result<impl IntoResponse, UserError> {
    let repo = state.user_repo.clone();
    let mut user = User::from(user_data);
    user.id = claims.sub;
    user.role = claims.role.into();

    if repo
        .update(repo.db_pool.clone().as_ref(), user)
        .await?
        .rows_affected()
        == 0
    {
        return Err(UserError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

// NOTE: если есть какая-то сложная ошибка связанная с handler-ом, но в handler-е ошибки нет, то
// можно добавить перед ним #[axum::debug_handler] и он покажет более подробную ошибку
#[utoipa::path(
    get,
    tag = "user",
    security(
        ("bearer_auth" = [])
    ),
    path = "/{offset}/{page_limit}",
    params(
        ("offset" = u64, Path, description = "Offset"),
        ("page_limit" = u64, Path, description = "Record limit")
    ),
    responses(
        (status = 200, description = "Users found", body = Vec<UserResponse>),
        (status = 204, description = "No users in this range"),
        (status = 500, description = "Internal database error", body = String)
    )
)]
pub async fn get_with_pagination(
    Path((offset, page_limit)): Path<(Offset, Limit)>,
    State(state): State<AppState>,
    // NOTE: с путем /{offset}/aboba/{page_limit} будет работать аналогчино
) -> Result<impl IntoResponse, UserError> {
    let repo = state.user_repo.clone();
    let users_vec = repo
        .get(&offset, &page_limit)
        .await?
        .into_iter()
        .map(UserResponse::from)
        .collect::<Vec<_>>();

    // NOTE: в случае кортежей разной длины в Ok надо делать .into_response()
    if users_vec.is_empty() {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((StatusCode::OK, Json(users_vec)).into_response())
    }
}
