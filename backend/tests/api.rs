use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceExt;

use reweave::{api, board, puzzle, words};

/// Create a basic test router with a small word list
fn create_test_router1() -> Router {
    let full_word_list = Arc::new(words::Trie::new(vec![
        "both", "broth", "foul", "trouble", "blur",
    ]));
    let all_puzzles = Arc::new(HashMap::new());

    api::router(full_word_list, all_puzzles)
}

/// Create a basic test router with a small word list and a puzzle
fn create_test_router2() -> Router {
    let full_word_list = Arc::new(words::Trie::new(vec![
        "bot", "hot", "tho", "too", "both", "hoot",
    ]));
    let all_puzzles = Arc::new(
        vec![(
            "test1".to_string(),
            puzzle::Puzzle::from_board(
                &board::Board::create(2, 2, vec!['o', 't', 'b', 'h']),
                &full_word_list,
            ),
        )]
        .into_iter()
        .collect(),
    );

    api::router(full_word_list, all_puzzles)
}

#[tokio::test]
async fn test_hello() {
    let app = create_test_router1();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/hello")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let message: api::Message = serde_json::from_slice(&body).unwrap();

    assert_eq!(message.text, "Hello from Rust");
}

#[tokio::test]
async fn test_greet() {
    let app = create_test_router1();

    let request_body = serde_json::to_string(&api::GreetInput {
        name: "Thomas".to_string(),
    })
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/greet")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let message: api::Message = serde_json::from_slice(&body).unwrap();

    assert_eq!(message.text, "Hello Thomas");
}

#[tokio::test]
async fn test_board() {
    let app = create_test_router1();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let mut found_words: Vec<String> = serde_json::from_slice(&body).unwrap();
    found_words.sort();

    assert_eq!(found_words, vec!["both", "broth", "foul"]);
}

#[tokio::test]
async fn test_find_from_board() {
    let app = create_test_router1();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/find?width=2&height=2&letters=otbh")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let found_words: Vec<String> = serde_json::from_slice(&body).unwrap();

    assert_eq!(found_words, vec!["both"]);
}

#[tokio::test]
async fn test_puzzle() {
    let app = create_test_router2();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/check-puzzle/test1/letters/hoot")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let mut words: puzzle::Words = serde_json::from_slice(&body).unwrap();

    words.found.sort();
    words.missing.sort();
    words.extra.sort();

    assert_eq!(
        words,
        puzzle::Words {
            found: vec!["hot".to_string(), "tho".to_string()],
            missing: vec!["bot".to_string(), "both".to_string()],
            extra: vec!["hoot".to_string(), "too".to_string()]
        }
    );
}

#[tokio::test]
async fn test_bad_puzzle_id() {
    let app = create_test_router2();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/check-puzzle/test2/letters/hoot")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_client_error());
}
