use serde_json::{Value, json};
use vercel_runtime::{Error, Request, run, service_fn};

use reweave::api::{LoadInput, load_puzzle};

async fn handler(req: Request) -> Result<Value, Error> {
    // Path for puzzle id
    let path = req.uri().path();
    let puzzle_id = path.split("/").last().unwrap_or("").to_string();

    match load_puzzle(LoadInput { puzzle_id }).await {
        Ok(out) => Ok(json!(out)),
        Err(e) => Ok(json!({"error": e.0})),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
