use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;
use vercel_runtime::Error;

use crate::common::puzzle;
use crate::db::*;

#[derive(Serialize)]
pub struct ErrorResponse(pub String);

/// From a helper method, build an output for an API endpoint
pub fn build_api_output<T: Serialize>(out: Result<T, ErrorResponse>) -> Result<Value, Error> {
    match out {
        Ok(out) => Ok(json!(out)),
        Err(e) => Ok(json!({ "error": e.0 })),
    }
}

#[derive(Deserialize)]
pub struct CreateInput {
    puzzle_id: String,
    width: usize,
    height: usize,
    letters: String,
    words: HashSet<String>,
}

pub async fn create(inp: CreateInput) -> Result<(), ErrorResponse> {
    let puzzle = match puzzle::Puzzle::create(inp.width, inp.height, inp.letters, inp.words) {
        Ok(puzzle) => puzzle,
        Err(error) => {
            // println!("Err: {}", msg);
            return Err(ErrorResponse(error));
        }
    };

    insert_puzzle_into_db(inp.puzzle_id, puzzle)
        .await
        .map_err(|e| ErrorResponse(e.to_string()))?;

    Ok(())
}

#[derive(Deserialize)]
pub struct LoadInput {
    pub puzzle_id: String,
}

pub async fn load_puzzle(inp: LoadInput) -> Result<puzzle::Puzzle, ErrorResponse> {
    match get_puzzle(&inp.puzzle_id).await {
        Some(puzzle) => Ok(puzzle.clone()),
        None => Err(ErrorResponse("invalid puzzle id".to_string())),
    }
}
