pub mod handlers;
pub mod implementations;
pub mod models;
pub mod schemas;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use crate::utilities::app_state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/profile", get(handlers::get_user_handler))
        .route("/api/v1/profile", patch(handlers::update_user_handler))
        .route("/api/v1/profile", delete(handlers::delete_user_handler))
        .route("/api/v1/logout", get(handlers::logout_handler))
        .route("/api/v1/auth/refresh", post(handlers::refresh_handler))
        .route("/api/v1/auth/logout", get(handlers::logout_handler))
        .route("/api/v1/auth/user", get(handlers::get_oauth_user_handler))
        // Unified email continue
        .route(
            "/api/v1/auth/email",
            post(handlers::continue_with_email_handler),
        )
        // Complete profile (generalized)
        .route(
            "/api/v1/auth/complete",
            patch(handlers::complete_profile_handler),
        )
        // Email verification
        .route("/api/v1/auth/verify", get(handlers::verify_handler))
        // Social logins
        .route("/api/v1/auth/google", get(handlers::google_oauth_handler))
        .route(
            "/api/v1/auth/google/callback",
            get(handlers::google_oauth_callback_handler),
        )
        .route("/api/v1/auth/github", get(handlers::github_oauth_handler))
        .route(
            "/api/v1/auth/github/callback",
            get(handlers::github_oauth_callback_handler),
        )
}
