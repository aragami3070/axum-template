mod config;
mod errors;
mod handlers;
mod models;
mod repository;
mod schemas;
mod services;

use axum::Router;
use axum::response::IntoResponse;
use axum::routing::*;
use dotenv::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use config::*;

use crate::handlers::auth::AuthDocs;
use crate::handlers::auth::AuthRouter;

#[derive(OpenApi)]
#[openapi(paths(aboba))]
pub struct AbobaDocs;
#[utoipa::path(
    get,
    path = "/aboba",
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

    let open_api = ApiDoc::openapi()
        .nest("/auth", AuthDocs::openapi())
        .nest("/aboba", AbobaDocs::openapi());

    let swagger_router = SwaggerUi::new("/docs").url("/api-docs/openapi.json", open_api);

    let auth_router = AuthRouter::set_router();
    let app = Router::new()
        .route("/aboba/aboba", get(aboba))
        .nest("/auth", auth_router)
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
