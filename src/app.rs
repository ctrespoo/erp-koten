use axum::{routing::get, Router};

use crate::modules::cadunico::routes;

pub fn build_app() -> Router {
    Router::new()
        .route("/", get(routes::home))
        .route("/cadunico", get(routes::index))
        .route("/cadunico/criar", get(routes::create))
}
