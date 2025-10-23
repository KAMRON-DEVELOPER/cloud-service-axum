use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use axum_extra::extract::PrivateCookieJar;
use object_store::gcp::GoogleCloudStorage;
use qdrant_client::Qdrant;
use serde_json::json;
use tracing::debug;
use uuid::Uuid;

use crate::{
    features::{
        listings::{
            repository::{create_listing, get_many_listings, get_one_listing},
            schemas::ListingResponse,
        },
        schemas::ListingQuery,
        users::schemas::RedirectResponse,
    },
    services::{ai::AI, database::Database},
    utilities::{cookie::OptionalOAuthUserIdCookie, errors::AppError, jwt::Claims},
};

pub async fn get_stats_handler(
    jar: PrivateCookieJar,
    State(database): State<Database>,
    OptionalOAuthUserIdCookie(optional_oauth_user_id_cookie): OptionalOAuthUserIdCookie,
) -> Result<Response, AppError> {
    if optional_oauth_user_id_cookie.is_some() {
        let response = Json(RedirectResponse {
            redirect_to: "complete-profile".to_string(),
        });
        return Ok((jar, response).into_response());
    }

    let total_users = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) from users
        "#
    )
    .fetch_one(&database.pool)
    .await?;
    let total_users = total_users.unwrap_or(0);

    let total_listings = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) from listings
        "#
    )
    .fetch_one(&database.pool)
    .await?;
    let total_listings = total_listings.unwrap_or(0);

    Ok(Json(json!({"totalUsers": total_users, "totalListings": total_listings})).into_response())
}

pub async fn get_many_listings_handler(
    jar: PrivateCookieJar,
    State(database): State<Database>,
    State(qdrant): State<Qdrant>,
    State(ai): State<AI>,
    Query(listing_query): Query<ListingQuery>,
    OptionalOAuthUserIdCookie(optional_oauth_user_id_cookie): OptionalOAuthUserIdCookie,
) -> Result<Response, AppError> {
    if optional_oauth_user_id_cookie.is_some() {
        let response = Json(RedirectResponse {
            redirect_to: "complete-profile".to_string(),
        });
        return Ok((jar, response).into_response());
    }

    listing_query.pagination.validate()?;

    debug!("country: {}", listing_query.search_params.country);

    let (listings, total) = get_many_listings(&database.pool, &listing_query, qdrant, ai).await?;

    Ok(Json(ListingResponse { listings, total }).into_response())
}

pub async fn get_one_listing_handler(
    State(database): State<Database>,
    Path(listing_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let listing = get_one_listing(&database.pool, &listing_id).await?;

    Ok(Json(listing))
}

pub async fn create_listing_handler(
    claims: Claims,
    State(database): State<Database>,
    State(ai): State<AI>,
    State(qdrant): State<Qdrant>,
    State(gcs): State<GoogleCloudStorage>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    create_listing(claims.sub, &database.pool, gcs, qdrant, ai, multipart).await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"message": "Your listing created successfully!"})),
    ))
}

pub async fn delete_listing_handler(
    claims: Claims,
    Path(listing_id): Path<Uuid>,
    State(database): State<Database>,
) -> Result<impl IntoResponse, AppError> {
    let query_result = sqlx::query!(
        "DELETE FROM listings where owner_id = $1 AND id = $2",
        claims.sub,
        listing_id
    )
    .execute(&database.pool)
    .await?;

    match query_result.rows_affected() {
        0 => Err(AppError::DatabaseDeleteError {
            resource: "Listing".to_string(),
            id: listing_id.to_string(),
        }),
        _ => Ok((
            StatusCode::NO_CONTENT,
            Json(json!({"message": "Your listing was deleted successfully!"})),
        )),
    }
}
