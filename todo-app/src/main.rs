use axum::{
    extract,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Route all requests on "/" endpoint to anonymous handler.
    //
    // A handler is an async function which returns something that implements
    // `axum::response::IntoResponse`.

    // A closure or a function can be used as handler.

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/json", post(json));

    // Address that server will bind to.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    // Use `hyper::server::Server` which is re-exported through `axum::Server` to serve the app.
    axum::Server::bind(&addr)
        // Hyper server takes a make service.
        .serve(app.into_make_service())
        .await
        .unwrap();
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

