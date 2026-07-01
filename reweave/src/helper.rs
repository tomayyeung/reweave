use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;
use vercel_runtime::{Error, Request, Response, ResponseBody};

use crate::common::puzzle;
use crate::db::*;

#[derive(Serialize)]
pub struct ErrorResponse(pub String);

/// Create a CORS response to OPTIONS method requests
pub fn cors_response(
    status: u16,
    body: impl Into<ResponseBody>,
) -> Result<Response<ResponseBody>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET,POST,OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(body.into())?)
}

/// Create a JSON response to most HTTP requests
pub fn json_response<T: Serialize>(
    out: Result<T, ErrorResponse>,
) -> Result<Response<ResponseBody>, Error> {
    // Status and value depend on Ok or Err
    let (status, value) = match out {
        Ok(val) => (200, json!(val)),
        Err(e) => (400, json!( {"error": e.0} )),
    };

    Ok(Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET,POST,OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(ResponseBody::from(value))?)
}

/// Create a JSON response with an error message
pub fn json_err_response(err: &str) -> Result<Response<ResponseBody>, Error> {
    json_response::<Value>(Err(ErrorResponse(String::from(err))))
}

/// Parse HTTP JSON body
pub async fn read_json_body<T: DeserializeOwned>(req: Request) -> Result<T, Error> {
    let bytes = req.into_body().collect().await?.to_bytes();
    Ok(serde_json::from_slice(&bytes)?)
}

#[derive(Deserialize)]
pub struct CreateInput {
    name: String,
    width: usize,
    height: usize,
    letters: String,
    words: HashSet<String>,
    answer: String,
}

#[derive(Serialize)]
pub struct CreateOutput {
    id: String,
}

pub async fn create(inp: CreateInput) -> Result<CreateOutput, ErrorResponse> {
    let puzzle = match puzzle::Puzzle::create(
        inp.name,
        inp.width,
        inp.height,
        inp.letters,
        inp.words,
        inp.answer,
    ) {
        Ok(puzzle) => puzzle,
        Err(error) => {
            return Err(ErrorResponse(error));
        }
    };

    let id = insert_puzzle_into_db(puzzle)
        .await
        .map_err(|e| ErrorResponse(e.to_string()))?;

    Ok(CreateOutput { id })
}

#[derive(Deserialize)]
pub struct LoadInput {
    pub puzzle_id: String,
}

pub async fn load_puzzle(inp: LoadInput) -> Result<puzzle::Puzzle, ErrorResponse> {
    match get_puzzle(&inp.puzzle_id).await {
        Some(puzzle) => Ok(puzzle),
        None => Err(ErrorResponse(format!(
            "invalid puzzle id: {}",
            &inp.puzzle_id
        ))),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleSummary {
    id: String,
    name: String,
    width: usize,
    height: usize,
    starting_letters: usize,
    total_cells: usize,
    given_percent: u8,
    description: Option<String>,
}

pub async fn list_puzzles() -> Result<Vec<PuzzleSummary>, ErrorResponse> {
    let records = list_puzzle_records()
        .await
        .map_err(|e| ErrorResponse(e.to_string()))?;

    Ok(records
        .into_iter()
        .map(|record| {
            let total_cells = record.width * record.height;
            let starting_letters = record
                .letters
                .chars()
                .filter(|letter| *letter != '_' && *letter != '!')
                .count();
            let given_percent = if total_cells == 0 {
                0
            } else {
                ((starting_letters * 100 + total_cells / 2) / total_cells) as u8
            };

            PuzzleSummary {
                id: record.id,
                name: record.name,
                width: record.width,
                height: record.height,
                starting_letters,
                total_cells,
                given_percent,
                description: None,
            }
        })
        .collect())
}
