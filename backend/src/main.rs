mod api;
mod board;
mod words;

#[tokio::main]
async fn main() {
    let app = api::router();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}