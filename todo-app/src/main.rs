use axum::{
    extract,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        .route("/json", post(json))
}

fn setup_middlewares(router: Router) -> Router {
    router.layer(TraceLayer::new_for_http()) // Tracing of each request
}

#[derive(Deserialize, Serialize, Debug)]
struct Todo {
    title: String,
    description: String,
}

// Json input parsing - and output
async fn json(extract::Json(payload): extract::Json<Todo>) -> Json<Todo> {
    Json(payload)
}
