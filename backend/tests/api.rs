use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

use reweave::{api, words};

fn create_test_router() -> Router {
    let full_word_list = Arc::new(words::Trie::new(vec![
        "both", "broth", "foul", "trouble", "blur",
    ]));
    api::router(full_word_list)
}

#[tokio::test]
async fn test_hello() {
    let app = create_test_router();

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
    let app = create_test_router();

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
    let app = create_test_router();

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
