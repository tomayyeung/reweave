use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    board::Board,
    words::{Trie, find_words},
};

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

async fn test_board(State(word_list): State<Arc<Trie>>) -> Json<Vec<String>> {
    let board = Arc::new(Board::create(
        3,
        3,
        vec!['t', 'r', 'b', 'h', 'o', 'u', 'f', 'l', 'y'],
    ));
    Json(find_words(&board, &word_list))
}

pub fn router(full_word_list: Arc<Trie>) -> Router {
    Router::new()
        .route("/api/hello", get(hello))
        .route("/api/greet", post(greet))
        .route("/api/test", get(test_board))
        .with_state(full_word_list)
}
