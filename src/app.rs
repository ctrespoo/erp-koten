use axum::{Router, routing::get};
use tower_http::services::ServeDir;

use crate::modules::cadunico::routes;
use crate::state::AppState;

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(routes::home))
        .route("/cadunico", get(routes::index).post(routes::submit))
        .route("/cadunico/criar", get(routes::create))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state)
}
