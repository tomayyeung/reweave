use std::sync::Arc;

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::words::Trie;

#[derive(Serialize)]
struct Message {
    text: String,
}

#[derive(Deserialize)]
struct Input {
    name: String,
}

/// This will be a GET endpoint
async fn hello() -> Json<Message> {
    Json(Message {
        text: "Hello from Rust".to_string(),
    })
}

/// This will be a POST endpoint
async fn greet(Json(input): Json<Input>) -> Json<Message> {
    Json(Message {
        text: format!("Hello {}", input.name),
    })
}

pub fn router(full_word_list: Arc<Trie>) -> Router {
    Router::new()
        .route("/api/hello", get(hello))
        .route("/api/greet", post(greet))
        .with_state(full_word_list)
}
