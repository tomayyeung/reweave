use serde_json::Value;
use vercel_runtime::{Error, Request};

use crate::helper::{CreateInput, build_api_output, create};

pub async fn handler(req: Request) -> Result<Value, Error> {
    let query = req.uri().query().unwrap_or("");

    let params: CreateInput = serde_urlencoded::from_str(query)
        .map_err(Box::<dyn std::error::Error + Send + Sync>::from)?;

    build_api_output(create(params).await)
}
