use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    board::{Board, find_words},
    puzzle::{Puzzle, Words},
    words::Trie,
};

#[derive(Clone)]
struct AppState {
    full_word_list: Arc<Trie>,
    all_puzzles: Arc<HashMap<String, Puzzle>>,
}

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

async fn test_board(State(state): State<AppState>) -> Json<Vec<String>> {
    let board = Arc::new(Board::create(
        3,
        3,
        vec!['t', 'r', 'b', 'h', 'o', 'u', 'f', 'l', 'y'],
    ));
    Json(find_words(&board, &state.full_word_list))
}

#[derive(Deserialize)]
pub struct BoardParam {
    width: usize,
    height: usize,
    letters: String,
}

async fn find_from_board(
    State(state): State<AppState>,
    Query(param): Query<BoardParam>,
) -> Json<Vec<String>> {
    // println!("{}", param.letters);
    Json(find_words(
        &Board::create(param.width, param.height, param.letters.chars().collect()),
        &state.full_word_list,
    ))
}

#[derive(Deserialize)]
pub struct PuzzleParam {
    letters: String,
    puzzle_id: String,
}

async fn puzzle(
    State(state): State<AppState>,
    Query(param): Query<PuzzleParam>,
    // ) -> Result<Json<HashMap<String, Vec<String>>>, (StatusCode, String)> {
) -> Result<Json<Words>, (StatusCode, &'static str)> {
    let Some(puzzle) = state.all_puzzles.get(&param.puzzle_id) else {
        return Err((StatusCode::BAD_REQUEST, "Invalid puzzle ID"));
    };

    let found_words = find_words(
        &Board::create(puzzle.width, puzzle.height, param.letters.chars().collect()),
        &state.full_word_list,
    );

    Ok(Json(puzzle.compare_found_words(found_words)))
}

#[derive(Deserialize)]
pub struct CreatePuzzleParam {
    puzzle_id: String,
    width: usize,
    height: usize,
    letters: String,
    words: HashSet<String>,
}

async fn create_puzzle(
    Json(param): Json<CreatePuzzleParam>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let puzzle = match Puzzle::create(
        param.width,
        param.height,
        param.letters.chars().collect(),
        param.words,
    ) {
        Ok(puzzle) => puzzle,
        Err(msg) => {
            return Err((StatusCode::BAD_REQUEST, msg));
        }
    };

    puzzle.to_file(format!("puzzles/{}.json", param.puzzle_id).as_str());

    Ok(StatusCode::OK)
}

pub fn router(full_word_list: Arc<Trie>, all_puzzles: Arc<HashMap<String, Puzzle>>) -> Router {
    let state = AppState {
        full_word_list,
        all_puzzles,
    };

    Router::new()
        .route("/api/hello", get(hello))
        .route("/api/greet", post(greet))
        .route("/api/test", get(test_board))
        .route("/api/find", get(find_from_board))
        .route("/api/puzzle", get(puzzle))
        .route("/api/create_puzzle", post(create_puzzle))
        .with_state(state)
}
