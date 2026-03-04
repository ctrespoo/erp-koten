use axum::{Router, routing::get};
use tower_http::services::ServeDir;

use crate::modules::cadunico::routes;

pub fn build_app() -> Router {
    Router::new()
        .route("/", get(routes::home))
        .route("/cadunico", get(routes::index))
        .route("/cadunico/criar", get(routes::create))
        .nest_service("/assets", ServeDir::new("assets"))
}
