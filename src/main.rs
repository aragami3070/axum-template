mod config;
mod errors;
mod handlers;
mod middlewares;
mod models;
mod repositories;
mod schemas;
mod services;

use axum::Router;
use axum::middleware::from_fn;
use axum::response::IntoResponse;
use axum::routing::*;
use dotenv::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa_swagger_ui::SwaggerUi;

use config::*;

use crate::handlers::auth::AuthDocs;
use crate::handlers::auth::AuthRouter;
use crate::handlers::users::{UserDocs, UserRouter};
use crate::middlewares::auth::auth_middleware;
use crate::middlewares::role::role_middleware;
use crate::models::users::Role;

#[derive(OpenApi)]
#[openapi(paths(aboba))]
pub struct AbobaDocs;
#[utoipa::path(
    get,
    path = "/aboba",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Aboba", body = String),
        (status = 500, description = "Технические шокаладки", body = String)
    )
)]
async fn aboba() -> impl IntoResponse {
    "aboba".to_string()
}

#[derive(OpenApi)]
#[openapi(info(title = "Blazingly-fast axum template API!!!", version = "1.0.0"))]
struct ApiDoc;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = Config::from_env();
    let db_pool = Arc::new(get_db_pool(&config.database_url).await);
    let state = AppState::new(
        db_pool,
        config.secret_key.to_owned(),
        config.secret_refresh_key.to_owned(),
    );
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));


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
    let swagger_router = SwaggerUi::new("/docs").url("/api-docs/openapi.json", open_api);


    let user_router = UserRouter::set_router(state.clone());
    let auth_router = AuthRouter::set_router();

    let protect_aboba_router = Router::new()
        .route("/aboba/aboba", get(aboba))
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

    let app = Router::new()
        .merge(protect_aboba_router)
        .nest("/auth", auth_router)
        .nest("/user", user_router)
        .merge(swagger_router)
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    println!("Listening on http://{}", &addr);
    println!("Swagger on http://{}/docs", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
