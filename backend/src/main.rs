use axum::{
    routing::get,
    Router,
    Json,
};
use serde::Serialize;
use tower_http::cors::CorsLayer;

#[derive(Serialize)]
struct Message {
    text: String
}

async fn hello() -> Json<Message> {
    Json(Message {
        text: "Hello from Rust!".to_string()
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/hello", get(hello))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}