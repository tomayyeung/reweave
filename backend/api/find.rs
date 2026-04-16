use serde_json::{Value, json};
use vercel_runtime::{Error, Request, run, service_fn};

use reweave::api::{FindInput, find};

async fn handler(req: Request) -> Result<Value, Error> {
    let query = req.uri().query().unwrap_or("");

    let params: FindInput = serde_urlencoded::from_str(query)
        .map_err(Box::<dyn std::error::Error + Send + Sync>::from)?;

    match find(params) {
        Ok(out) => Ok(json!(out)),
        Err(e) => Ok(json!({ "error": e.0 })),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
