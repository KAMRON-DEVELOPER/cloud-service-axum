pub mod handlers;
pub mod implementations;
pub mod models;
pub mod repository;
pub mod schemas;
pub mod websocket;

use crate::utilities::app_state::AppState;

use axum::{
    Router,
    routing::{delete, get, patch},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/projects",
            get(handlers::get_projects).post(handlers::create_project),
        )
        .route("/api/v1/project/:id", get(handlers::get_project))
        .route("/api/v1/project/:id", patch(handlers::update_project))
        .route("/api/v1/project/:id", delete(handlers::delete_project))
}
