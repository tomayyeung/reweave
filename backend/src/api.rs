// use std::{
//     collections::{HashMap, HashSet},
//     sync::Arc,
// };

// use axum::{
//     Json, Router,
//     extract::{Path, Query, State},
//     http::StatusCode,
//     routing::{get, post},
// };
// use serde::Deserialize;

// use crate::{
//     board::{Board, find_words},
//     puzzle::{Puzzle, Words},
//     words::Trie,
// };

// #[derive(Clone)]
// struct AppState {
//     full_word_list: Arc<Trie>,
//     puzzle_path: Arc<String>,
//     all_puzzles: Arc<HashMap<String, Puzzle>>,
// }

// #[derive(Deserialize)]
// pub struct BoardParam {
//     width: usize,
//     height: usize,
//     letters: String,
// }

// async fn find_from_board(
//     State(state): State<AppState>,
//     Query(param): Query<BoardParam>,
// ) -> Result<(StatusCode, Json<Vec<String>>), (StatusCode, String)> {
//     // println!("{}", param.letters);
//     let board = match Board::create(param.width, param.height, param.letters.chars().collect()) {
//         Ok(board) => board,
//         Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
//     };

//     Ok((
//         StatusCode::OK,
//         Json(find_words(&board, &state.full_word_list)),
//     ))
// }

// #[derive(Deserialize)]
// pub struct CheckPuzzleParams {
//     letters: String,
//     puzzle_id: String,
// }

// async fn check_puzzle(
//     State(state): State<AppState>,
//     Path(param): Path<CheckPuzzleParams>,
// ) -> Result<(StatusCode, Json<Words>), (StatusCode, String)> {
//     let Some(puzzle) = state.all_puzzles.get(&param.puzzle_id) else {
//         return Err((StatusCode::BAD_REQUEST, "Invalid puzzle ID".to_string()));
//     };

//     let board = match Board::create(puzzle.width, puzzle.height, param.letters.chars().collect()) {
//         Ok(board) => board,
//         Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
//     };

//     let found_words = find_words(&board, &state.full_word_list);

//     Ok((
//         StatusCode::OK,
//         Json(puzzle.compare_found_words(found_words)),
//     ))
// }

// #[derive(Deserialize)]
// pub struct CreatePuzzleParams {
//     puzzle_id: String,
//     width: usize,
//     height: usize,
//     letters: String,
//     words: HashSet<String>,
// }

// async fn create_puzzle(
//     State(state): State<AppState>,
//     Json(param): Json<CreatePuzzleParams>,
// ) -> Result<StatusCode, (StatusCode, &'static str)> {
//     let puzzle = match Puzzle::create(param.width, param.height, param.letters, param.words) {
//         Ok(puzzle) => puzzle,
//         Err(msg) => {
//             // println!("Err: {}", msg);
//             return Err((StatusCode::BAD_REQUEST, msg));
//         }
//     };

//     puzzle.to_file(format!("{}/{}.json", state.puzzle_path, param.puzzle_id).as_str());

//     Ok(StatusCode::OK)
// }

// async fn load_puzzle(
//     State(state): State<AppState>,
//     Path(puzzle_id): Path<String>,
// ) -> Result<(StatusCode, Json<Puzzle>), (StatusCode, &'static str)> {
//     match state.all_puzzles.get(&puzzle_id) {
//         Some(p) => Ok((StatusCode::OK, Json(p.clone()))),
//         None => Err((StatusCode::BAD_REQUEST, "Invalid puzzle ID")),
//     }
// }

// pub fn router(full_word_list: Arc<Trie>, all_puzzles: Arc<HashMap<String, Puzzle>>, puzzle_path: Arc<String>) -> Router {
//     let state = AppState {
//         full_word_list,
//         all_puzzles,
//         puzzle_path,
//     };

//     Router::new()
//         .route("/api/find", get(find_from_board))
//         .route(
//             "/api/check-puzzle/:puzzle_id/letters/:letters",
//             get(check_puzzle),
//         )
//         .route("/api/puzzle", post(create_puzzle))
//         .route("/api/puzzle/:puzzle_id", get(load_puzzle))
//         .with_state(state)
// }

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
// use serde_json::json;
// use vercel_runtime::Response;
// use axum::body::Body;
// use http::StatusCode;

use crate::{data::{board, puzzle}, get_puzzle};

#[derive(Serialize)]
pub struct ErrorResponse(pub String);

// /// Build an error HTTP response
// pub fn build_error(e: String) -> Response<Body> {
//     Response::builder()
//         .status(StatusCode::BAD_REQUEST)
//         .header("Content-Type", "application/json")
//         .body(Body::from(json!({"error": e}).to_string()))
//         .unwrap()
// }

// /// Build HTTP response based on result of an API request
// pub fn build_response<T: Serialize>(res: Result<T, ErrorResponse>) -> Response<Body> {
//     match res {
//         Ok(output) => Response::builder()
//             .status(StatusCode::OK)
//             .header("Content-Type", "application/json")
//             .body(Body::from(
//                 serde_json::to_string(&output).unwrap_or_default(),
//             ))
//             .unwrap(),
//         Err(e) => build_error(e.0)
//     }
// }

#[derive(Deserialize)]
pub struct FindInput {
    width: usize,
    height: usize,
    letters: String,
}

#[derive(Serialize)]
pub struct FindOutput {
    words: Vec<String>,
}

pub fn find(inp: FindInput) -> Result<FindOutput, ErrorResponse> {
    let board = match board::Board::create(inp.width, inp.height, inp.letters.chars().collect()) {
        Ok(board) => board,
        Err(error) => {
            return Err(ErrorResponse(error));
        }
    };

    Ok(FindOutput {
        words: board::find_words(&board, super::get_words()),
    })
}

#[derive(Deserialize)]
pub struct CheckInput {
    pub letters: String,
    pub puzzle_id: String,
}

#[derive(Serialize)]
pub struct CheckOutput {
    words: puzzle::Words,
}

pub async fn check_puzzle(inp: CheckInput) -> Result<CheckOutput, ErrorResponse> {
    let puzzle = match get_puzzle(&inp.puzzle_id).await {
        Some(puzzle) => puzzle,
        None => return Err(ErrorResponse("invalid puzzle id".to_string())),
    };

    let board =
        match board::Board::create(puzzle.width, puzzle.height, inp.letters.chars().collect()) {
            Ok(board) => board,
            Err(error) => return Err(ErrorResponse(error)),
        };

    let found_words = board::find_words(&board, super::get_words());

    Ok(CheckOutput {
        words: puzzle.compare_found_words(found_words),
    })
}

#[derive(Deserialize)]
pub struct CreateInput {
    puzzle_id: String,
    width: usize,
    height: usize,
    letters: String,
    words: HashSet<String>,
}

pub async fn create(
    inp: CreateInput,
) -> Result<(), ErrorResponse> {
    let puzzle = match puzzle::Puzzle::create(inp.width, inp.height, inp.letters, inp.words) {
        Ok(puzzle) => puzzle,
        Err(error) => {
            // println!("Err: {}", msg);
            return Err(ErrorResponse(error));
        }
    };

    super::insert_puzzle_into_db(inp.puzzle_id, puzzle)
        .await.map_err(|e| ErrorResponse(e.to_string()))?;

    Ok(())
}

#[derive(Deserialize)]
pub struct LoadInput {
    pub puzzle_id: String,
}

pub async fn load_puzzle(
    inp: LoadInput,
) -> Result<puzzle::Puzzle, ErrorResponse> {
    match super::get_puzzle(&inp.puzzle_id).await {
        Some(puzzle) => Ok(puzzle.clone()),
        None => return Err(ErrorResponse("invalid puzzle id".to_string())),
    }
}