use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;

use crate::{
    board::{Board, find_words},
    puzzle::{Puzzle, Words},
    words::Trie,
};

#[derive(Clone)]
struct AppState {
    full_word_list: Arc<Trie>,
    puzzle_path: Arc<String>,
    all_puzzles: Arc<HashMap<String, Puzzle>>,
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
) -> Result<(StatusCode, Json<Vec<String>>), (StatusCode, String)> {
    // println!("{}", param.letters);
    let board = match Board::create(param.width, param.height, param.letters.chars().collect()) {
        Ok(board) => board,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
    };

    Ok((
        StatusCode::OK,
        Json(find_words(&board, &state.full_word_list)),
    ))
}

#[derive(Deserialize)]
pub struct CheckPuzzleParams {
    letters: String,
    puzzle_id: String,
}

async fn check_puzzle(
    State(state): State<AppState>,
    Path(param): Path<CheckPuzzleParams>,
) -> Result<(StatusCode, Json<Words>), (StatusCode, String)> {
    let Some(puzzle) = state.all_puzzles.get(&param.puzzle_id) else {
        return Err((StatusCode::BAD_REQUEST, "Invalid puzzle ID".to_string()));
    };

    let board = match Board::create(puzzle.width, puzzle.height, param.letters.chars().collect()) {
        Ok(board) => board,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
    };

    let found_words = find_words(&board, &state.full_word_list);

    Ok((
        StatusCode::OK,
        Json(puzzle.compare_found_words(found_words)),
    ))
}

#[derive(Deserialize)]
pub struct CreatePuzzleParams {
    puzzle_id: String,
    width: usize,
    height: usize,
    letters: String,
    words: HashSet<String>,
}

async fn create_puzzle(
    State(state): State<AppState>,
    Json(param): Json<CreatePuzzleParams>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let puzzle = match Puzzle::create(param.width, param.height, param.letters, param.words) {
        Ok(puzzle) => puzzle,
        Err(msg) => {
            // println!("Err: {}", msg);
            return Err((StatusCode::BAD_REQUEST, msg));
        }
    };

    puzzle.to_file(format!("{}/{}.json", state.puzzle_path, param.puzzle_id).as_str());

    Ok(StatusCode::OK)
}

async fn load_puzzle(
    State(state): State<AppState>,
    Path(puzzle_id): Path<String>,
) -> Result<(StatusCode, Json<Puzzle>), (StatusCode, &'static str)> {
    match state.all_puzzles.get(&puzzle_id) {
        Some(p) => Ok((StatusCode::OK, Json(p.clone()))),
        None => Err((StatusCode::BAD_REQUEST, "Invalid puzzle ID")),
    }
}

pub fn router(full_word_list: Arc<Trie>, all_puzzles: Arc<HashMap<String, Puzzle>>, puzzle_path: Arc<String>) -> Router {
    let state = AppState {
        full_word_list,
        all_puzzles,
        puzzle_path,
    };

    Router::new()
        .route("/api/find", get(find_from_board))
        .route(
            "/api/check-puzzle/:puzzle_id/letters/:letters",
            get(check_puzzle),
        )
        .route("/api/puzzle", post(create_puzzle))
        .route("/api/puzzle/:puzzle_id", get(load_puzzle))
        .with_state(state)
}
