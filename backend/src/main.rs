use std::sync::Arc;

use reweave::{api, words};

const WORDS: &str = include_str!("../../wordlist/wordlist.txt");

#[tokio::main]
async fn main() {
    // Initialize words
    let full_word_list = Arc::new(words::Trie::new(
        WORDS.split("\n").collect()
    ));

    // Initialize APIs
    let app = api::router(full_word_list);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
