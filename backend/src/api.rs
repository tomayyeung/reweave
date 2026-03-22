use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    board::{Board, find_words},
    words::Trie,
};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct GreetInput {
    pub name: String,
}

/// This will be a GET endpoint
async fn hello() -> Json<Message> {
    Json(Message {
        text: "Hello from Rust".to_string(),
    })
}

/// This will be a POST endpoint
async fn greet(Json(input): Json<GreetInput>) -> Json<Message> {
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

#[derive(Deserialize)]
pub struct BoardParam {
    width: usize,
    height: usize,
    letters: String,
}

async fn find_from_board(
    State(word_list): State<Arc<Trie>>,
    Query(param): Query<BoardParam>,
) -> Json<Vec<String>> {
    Json(find_words(
        &Board::create(param.width, param.height, param.letters.chars().collect()),
        &word_list,
    ))
}

pub fn router(full_word_list: Arc<Trie>) -> Router {
    Router::new()
        .route("/api/hello", get(hello))
        .route("/api/greet", post(greet))
        .route("/api/test", get(test_board))
        .route("/api/find", get(find_from_board))
        .with_state(full_word_list)
}
