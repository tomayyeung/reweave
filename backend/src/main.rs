use std::sync::Arc;

mod api;
mod board;
mod words;

#[tokio::main]
async fn main() {
    // Initialize words
    let full_word_list = Arc::new(words::Trie::new(vec!["abs", "abacus", "teeth", "tusk"]));

    // Initialize APIs
    let app = api::router(full_word_list);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
