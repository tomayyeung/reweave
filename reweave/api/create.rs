use serde_json::Value;
use vercel_runtime::{Error, Request};

use reweave::helper::{CreateInput, build_api_output, create};
use vercel_runtime::{run, service_fn};

pub async fn handler(req: Request) -> Result<Value, Error> {
    let query = req.uri().query().unwrap_or("");

    let params: CreateInput = serde_urlencoded::from_str(query)
        .map_err(Box::<dyn std::error::Error + Send + Sync>::from)?;

    build_api_output(create(params).await)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
