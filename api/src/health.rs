use serde_json::{Value, json};
use vercel_runtime::{Error, Request};

use crate::helper::{build_api_output};

pub async fn handler(_req: Request) -> Result<Value, Error> {
    build_api_output(Ok(json!("ok")))
}
