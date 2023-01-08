use axum::{http, middleware::Next, response::Response};
use hyper::{Request, StatusCode};

pub async fn header_auth<B>(req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(auth_header) if auth_header == "GBA" => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
