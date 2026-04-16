use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{
    data::{board, puzzle},
    get_puzzle,
};

#[derive(Serialize)]
pub struct ErrorResponse(pub String);

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

pub async fn create(inp: CreateInput) -> Result<(), ErrorResponse> {
    let puzzle = match puzzle::Puzzle::create(inp.width, inp.height, inp.letters, inp.words) {
        Ok(puzzle) => puzzle,
        Err(error) => {
            // println!("Err: {}", msg);
            return Err(ErrorResponse(error));
        }
    };

    super::insert_puzzle_into_db(inp.puzzle_id, puzzle)
        .await
        .map_err(|e| ErrorResponse(e.to_string()))?;

    Ok(())
}

#[derive(Deserialize)]
pub struct LoadInput {
    pub puzzle_id: String,
}

pub async fn load_puzzle(inp: LoadInput) -> Result<puzzle::Puzzle, ErrorResponse> {
    match super::get_puzzle(&inp.puzzle_id).await {
        Some(puzzle) => Ok(puzzle.clone()),
        None => Err(ErrorResponse("invalid puzzle id".to_string())),
    }
}
