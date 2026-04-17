use axum::{Router, middleware::from_fn, response::IntoResponse, routing::get};
use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(info(title = "Blazingly-fast axum template API!!!", version = "1.0.0"))]
struct ApiDoc;

use crate::{
    config::AppState,
    handlers::{
        auth::{AuthDocs, AuthRouter},
        users::{UserDocs, UserRouter},
    }, middlewares::{auth::auth_middleware, role::role_middleware}, models::users::Role,
};

pub fn get_swagger_routes() -> SwaggerUi {
    // NOTE: сюда добавляем доки для swagger-а
    let mut open_api = ApiDoc::openapi()
        .nest("/auth", AuthDocs::openapi())
        .nest("/user", UserDocs::openapi())
        .nest("/aboba", AbobaDocs::openapi());

    // NOTE: добавляет подстановку токена в запрос
    open_api
        .components
        .get_or_insert_with(Default::default)
        .add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    SwaggerUi::new("/docs").url("/api-docs/openapi.json", open_api)
}

pub fn get_all_routes(state: AppState) -> Router<AppState> {
    // NOTE: здесь создаете свои router-ы
    let user_router = UserRouter::set_router(state.clone());
    let auth_router = AuthRouter::set_router();

    let protect_aboba_router = Router::new()
        .route("/abobus", get(aboba))
        .route_layer(from_fn(move |req, next| async move {
            role_middleware(req, next, Role::all()).await
        }))
        .route_layer({
            let token_serv = state.token_serv.clone();
            from_fn(move |req, next| {
                let token_serv = token_serv.clone();
                async move { auth_middleware(req, next, token_serv.clone()).await }
            })
        });

    // NOTE: здесь добавлять в ко всем router-ам
    Router::new()
        .nest("/aboba", protect_aboba_router)
        .nest("/auth", auth_router)
        .nest("/user", user_router)
}




#[derive(OpenApi)]
#[openapi(paths(aboba))]
pub struct AbobaDocs;
#[utoipa::path(
    get,
    path = "/abobus",
    security(
        ("bearer_auth" = [])
    ),
    tag = "aboba",
    responses(
        (status = 200, description = "Aboba", body = String),
        (status = 500, description = "Технические шокаладки", body = String)
    )
)]
async fn aboba() -> impl IntoResponse {
    "aboba".to_string()
}
