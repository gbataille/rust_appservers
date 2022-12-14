use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const MAX_CONTENT_LENGTH_BYTES: u64 = 1024 * 1024 * 2; // 100 MiB

pub mod authentication;
pub mod controllers;
pub mod models;
pub mod security;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "todo-app=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // logger sample
    tracing::debug!(target: "todo-app", address = addr.to_string(), "listening on {}", addr);

    // Use `hyper::server::Server` which is re-exported through `axum::Server` to serve the app.
    axum::Server::bind(&addr)
        // Hyper server takes a make service.
        .serve(app().into_make_service())
        .await
        .unwrap();
}

fn app() -> Router {
    let router = Router::new();
    let router = setup_routes(router);
    let router = setup_middlewares(router);
    router
}

fn setup_routes(router: Router) -> Router {
    router
        .route("/", get(|| async { "Hello, world!" }))
        .route("/404", get(controllers::return_404))
        .route("/500", get(|| async { "500" }))
        .route("/json", post(controllers::json))
}

fn setup_middlewares(router: Router) -> Router {
    router
        .layer(TraceLayer::new_for_http()) // Tracing of each request
        // Block request with crazy content length https://rustsec.org/advisories/RUSTSEC-2022-0055.html
        .route_layer(middleware::from_fn(security::content_length_guard))
        // Only allow request with the Authorization heador set to GBA
        .route_layer(middleware::from_fn(authentication::header_auth))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use serde_json::{json, Value};
    use tower::ServiceExt; // for `oneshot` and `ready`

    #[tokio::test]
    async fn test_hello_world() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(http::header::AUTHORIZATION, "GBA")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"Hello, world!");
    }

    #[tokio::test]
    async fn test_json() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/json")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .header(http::header::AUTHORIZATION, "GBA")
                    .body(Body::from(
                        serde_json::to_vec(
                            &json!({"title": "foo", "description": "Bar ja bulle!"}),
                        )
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            body,
            json!({ "title": "foo", "description": "Bar ja bulle!"})
        );
    }

    #[tokio::test]
    async fn test_not_found() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/does-not-exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(body.is_empty());
    }

    // // You can also spawn a server and talk to it like any other HTTP server:
    // #[tokio::test]
    // async fn the_real_deal() {
    //     let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    //     let addr = listener.local_addr().unwrap();

    //     tokio::spawn(async move {
    //         axum::Server::from_tcp(listener)
    //             .unwrap()
    //             .serve(app().into_make_service())
    //             .await
    //             .unwrap();
    //     });

    //     let client = hyper::Client::new();

    //     let response = client
    //         .request(
    //             Request::builder()
    //                 .uri(format!("http://{}", addr))
    //                 .body(Body::empty())
    //                 .unwrap(),
    //         )
    //         .await
    //         .unwrap();

    //     let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    //     assert_eq!(&body[..], b"Hello, World!");
    // }

    // // You can use `ready()` and `call()` to avoid using `clone()`
    // // in multiple request
    // #[tokio::test]
    // async fn multiple_request() {
    //     let mut app = app();

    //     let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    //     let response = app.ready().await.unwrap().call(request).await.unwrap();
    //     assert_eq!(response.status(), StatusCode::OK);

    //     let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    //     let response = app.ready().await.unwrap().call(request).await.unwrap();
    //     assert_eq!(response.status(), StatusCode::OK);
    // }
}
