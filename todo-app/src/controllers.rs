use axum::{extract, Json};
use hyper::StatusCode;

pub async fn return_404() -> Result<(), StatusCode> {
    Err(StatusCode::NOT_FOUND)
}

// Json input parsing - and output
pub async fn json(
    extract::Json(payload): extract::Json<super::models::todo::Todo>,
) -> Json<super::models::todo::Todo> {
    Json(payload)
}
