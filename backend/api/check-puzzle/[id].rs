use serde_json::{Value, json};
use std::collections::HashMap;
use vercel_runtime::{Error, Request, run, service_fn};

use reweave::api::{CheckInput, check_puzzle};

async fn handler(req: Request) -> Result<Value, Error> {
    // Path for puzzle id
    let path = req.uri().path();
    let puzzle_id = path.split("/").last().unwrap_or("").to_string();

    // Query for letters
    let query = req.uri().query().unwrap_or("");
    let query_params: HashMap<&str, String> = serde_urlencoded::from_str(query)
        .map_err(Box::<dyn std::error::Error + Send + Sync>::from)?;
    let letters = query_params
        .get("letters")
        .ok_or("Invalid letters input")?
        .clone();

    match check_puzzle(CheckInput { letters, puzzle_id }).await {
        Ok(out) => Ok(json!(out)),
        Err(e) => Ok(json!({"error": e.0})),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
