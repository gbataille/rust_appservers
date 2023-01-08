use axum::{http, middleware::Next, response::Response};
use hyper::{Request, StatusCode};

/// Prevents DoS through a fake Content-Length that would create a huge (impossible) memory
/// allocation
/// ```
/// curl --verbose --json '{"title":"foo", "description":"bar"}'  -X POST http://localhost:3000/json --header 'Content-Length:999999999999' --header 'Authorization:GBA'
/// ```
pub async fn content_length_guard<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let content_length = req
        .headers()
        .get(http::header::CONTENT_LENGTH)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.parse::<u64>().ok());

    tracing::debug!(target: "todo-app", "Content-Length {:?}", content_length);

    if content_length.is_some() && content_length.unwrap() > super::MAX_CONTENT_LENGTH_BYTES {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(next.run(req).await)
}
