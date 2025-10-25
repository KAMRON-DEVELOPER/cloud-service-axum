pub mod handlers;
pub mod implementations;
pub mod models;
pub mod repository;
pub mod schemas;

use crate::utilities::app_state::AppState;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/projects", get(handlers::get_projects))
        .route("/api/v1/project:project_id", get(handlers::get_project))
        .route("/api/v1/project:project_id", post(handlers::create_project))
        .route(
            "/api/v1/project:project_id",
            patch(handlers::update_project),
        )
        .route(
            "/api/v1/project:project_id",
            delete(handlers::delete_project),
        )
}
