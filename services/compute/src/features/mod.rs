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
        // Projects
        .route(
            "/api/v1/projects",
            get(handlers::get_projects).post(handlers::create_project),
        )
        .route(
            "/api/v1/project/:id",
            get(handlers::get_project)
                .patch(handlers::update_project)
                .delete(handlers::delete_project),
        )
        // Deployments
        .route(
            "/api/v1/project/:project_id/deployments",
            get(handlers::get_deployments).post(handlers::create_deployment),
        )
        .route(
            "/api/v1/deployment/:id",
            get(handlers::get_deployment)
                .patch(handlers::scale_deployment)
                .delete(handlers::delete_deployment),
        )
}
