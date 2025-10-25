use axum::response::IntoResponse;
use shared::utilities::errors::AppError;

pub async fn get_balance() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub async fn update_balance() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub async fn delete_balance() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub async fn cerate_balance() -> Result<impl IntoResponse, AppError> {
    Ok(())
}
