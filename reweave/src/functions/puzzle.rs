// use serde_json::Value;
// use vercel_runtime::{Error, Request};

// use crate::helper::{LoadInput, build_api_output, load_puzzle};
// // use backend::helper::{load_puzzle, LoadInput, build_api_output};
// use vercel_runtime::{run, service_fn};

// pub async fn handler(req: Request) -> Result<Value, Error> {
//     // Path for puzzle id
//     let path = req.uri().path();
//     let puzzle_id = path.split("/").last().unwrap_or("").to_string();

//     build_api_output(load_puzzle(LoadInput { puzzle_id }).await)
// }

// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     run(service_fn(handler)).await
// }
