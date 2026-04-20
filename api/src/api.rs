use serde_json::{json, Value};
use vercel_runtime::{Error, Request, run, service_fn};

use backend::{create, health, puzzle};

async fn handler(req: Request) -> Result<Value, Error> {
    let path = req.uri().path();
    match path {
        "/api/health" => health::handler(req).await,
        "/api/create" => create::handler(req).await,
        p if p.starts_with("/api/puzzle/") => puzzle::handler(req).await,
        _ => Ok(json!({ "error": "not found" }))
    }
}

// vc dev is shit: cargo build --bin api && ./target/debug/api
#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
