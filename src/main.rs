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
use sqlx::Pool;
use sqlx::Postgres;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use config::*;

use crate::handlers::auth::AuthDocs;
use crate::handlers::auth::AuthRouter;
async fn aboba() -> impl IntoResponse {
    "aboba".to_string()
}

#[derive(OpenApi)]
#[openapi(info(title = "Blazingly-fast axum template API!!!", version = "1.0.0"))]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<Pool<Postgres>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = Config::from_env();
    let db_pool = Arc::new(get_db_pool(&config.database_url).await);
    let state = AppState { db_pool };

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    let open_api = ApiDoc::openapi().nest("/auth", AuthDocs::openapi());
    let swagger_router = SwaggerUi::new("/docs").url("/api-docs/openapi.json", open_api);

    let auth_router = AuthRouter::set_router().route("/aboba", get(aboba));

    let app = Router::new()
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
