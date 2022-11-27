use axum::{
    extract, response,
    routing::{get, post},
    Json, Router,
};
use rand::{thread_rng, Rng};
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
        .route("/html", get(html))
        .route("/json", post(json))
        .route("/rand", get(rand));

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

// `Deserialize` need be implemented to use with `Query` extractor.
#[derive(Deserialize)]
struct RangeParameters {
    start: usize,
    end: usize,
}

// Query parameters parsing
async fn rand(extract::Query(range): extract::Query<RangeParameters>) -> response::Html<String> {
    // Generate a random number in range parsed from query.
    let random_number = thread_rng().gen_range(range.start..range.end);

    // Send response in html format.
    response::Html(format!("<h1>Random Number: {}</h1>", random_number))
}

// Returning HTML
async fn html() -> response::Html<&'static str> {
    // `std::include_str` macro can be used to include an utf-8 file as `&'static str` in compile
    // time. This method is relative to current `main.rs` file.
    response::Html(include_str!("../index.html"))
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
