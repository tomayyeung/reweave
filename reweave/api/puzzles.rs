use vercel_runtime::{Error, Request, Response, ResponseBody, run, service_fn};

use reweave::helper::{cors_response, json_err_response, json_response, list_puzzles};

pub async fn handler(req: Request) -> Result<Response<ResponseBody>, Error> {
    match req.method().as_str() {
        "OPTIONS" => cors_response(204, ""),
        "GET" => json_response(list_puzzles().await),
        _ => json_err_response("Invalid method request"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
