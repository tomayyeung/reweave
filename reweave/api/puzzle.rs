use serde_json::Value;
use vercel_runtime::{Error, Request};

use reweave::helper::{LoadInput, build_api_output, load_puzzle};
use vercel_runtime::{run, service_fn};

pub async fn handler(req: Request) -> Result<Value, Error> {
    // // Path for puzzle id
    // let path = req.uri().path();
    // let puzzle_id = path.split("/").last().unwrap_or("").to_string();
    let query = req.uri().query().unwrap_or("");
    let params: LoadInput = serde_urlencoded::from_str(query)
        .map_err(Box::<dyn std::error::Error + Send + Sync>::from)?;


    build_api_output(load_puzzle(params).await)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
