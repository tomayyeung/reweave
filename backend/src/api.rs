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
) -> Json<Vec<String>> {
    // println!("{}", param.letters);
    Json(find_words(
        &Board::create(param.width, param.height, param.letters.chars().collect()),
        &state.full_word_list,
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
) -> Result<(StatusCode, Json<Words>), (StatusCode, &'static str)> {
    let Some(puzzle) = state.all_puzzles.get(&param.puzzle_id) else {
        return Err((StatusCode::BAD_REQUEST, "Invalid puzzle ID"));
    };

    let found_words = find_words(
        &Board::create(puzzle.width, puzzle.height, param.letters.chars().collect()),
        &state.full_word_list,
    );

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
    Json(param): Json<CreatePuzzleParams>,
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

async fn load_puzzle(
    State(state): State<AppState>,
    Path(puzzle_id): Path<String>,
) -> Result<(StatusCode, Json<Puzzle>), (StatusCode, &'static str)> {
    match state.all_puzzles.get(&puzzle_id) {
        Some(p) => Ok((StatusCode::OK, Json(p.clone()))),
        None => Err((StatusCode::BAD_REQUEST, "Invalid puzzle ID")),
    }
}

pub fn router(full_word_list: Arc<Trie>, all_puzzles: Arc<HashMap<String, Puzzle>>) -> Router {
    let state = AppState {
        full_word_list,
        all_puzzles,
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
