use serde_json::{Value, json};
use vercel_runtime::{Error, Request};

use reweave::helper::build_api_output;
use vercel_runtime::{run, service_fn};

pub async fn handler(_req: Request) -> Result<Value, Error> {
    build_api_output(Ok(json!("ok")))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
