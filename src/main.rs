mod config;
mod errors;
mod handlers;
mod middlewares;
mod models;
mod repositories;
mod routes;
mod schemas;
mod services;

use axum::Router;
use axum::middleware::from_fn;
use axum::response::IntoResponse;
use axum::routing::*;
use dotenv::dotenv;
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;

use config::*;

use crate::handlers::auth::AuthRouter;
use crate::handlers::users::UserRouter;
use crate::middlewares::auth::auth_middleware;
use crate::middlewares::role::role_middleware;
use crate::models::users::Role;
use crate::routes::get_swagger_routes;

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

    let swagger_router = get_swagger_routes();

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

    let app = Router::new()
        .nest("/aboba", protect_aboba_router)
        .nest("/auth", auth_router)
        .nest("/user", user_router)
        .merge(swagger_router)
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT]),
        );

    println!("Listening on http://{}", &addr);
    println!("Swagger on http://{}/docs", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
