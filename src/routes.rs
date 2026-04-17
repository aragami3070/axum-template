use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(info(title = "Blazingly-fast axum template API!!!", version = "1.0.0"))]
struct ApiDoc;

use crate::{
    AbobaDocs,
    handlers::{auth::AuthDocs, users::UserDocs},
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
