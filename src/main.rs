mod config;
mod traicing;
mod errors;
mod handlers;
mod middlewares;
mod models;
mod repositories;
mod routes;
mod schemas;
mod services;

use axum::Router;
use dotenv::dotenv;
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use config::*;

use crate::{routes::{get_all_routes, get_swagger_routes}, traicing::init_tracing};

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_tracing();

    let config = Config::from_env();
    let db_pool = Arc::new(get_db_pool(&config.database_url).await);
    let state = AppState::new(
        db_pool,
        config.secret_key.to_owned(),
        config.secret_refresh_key.to_owned(),
    );
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    let swagger_router = get_swagger_routes();

    let all_routers = get_all_routes(state.clone());

    let app = Router::new()
        .merge(all_routers)
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
