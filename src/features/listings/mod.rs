// pub mod handlers;
// pub mod models;
// pub mod repository;
// pub mod schemas;

// use axum::{
//     Router,
//     routing::{delete, get},
// };

// use crate::utilities::app_state::AppState;

// pub fn routes() -> Router<AppState> {
//     Router::new()
//         .route(
//             "/api/v1/listings",
//             get(handlers::get_many_listings_handler).post(handlers::create_listing_handler),
//         )
//         .route("/api/v1/listings/stats", get(handlers::get_stats_handler))
//         .route(
//             "/api/v1/listings/{id}",
//             get(handlers::get_one_listing_handler).delete(handlers::delete_listing_handler),
//         )
//         .route("/api/v1/listings", delete(handlers::delete_listing_handler))
// }
