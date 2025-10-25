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
        .route("/api/v1/profile", get(handlers::get_balance))
        .route("/api/v1/profile", patch(handlers::update_balance))
        .route("/api/v1/profile", delete(handlers::delete_balance))
        .route("/api/v1/auth/refresh", post(handlers::cerate_balance))
}
