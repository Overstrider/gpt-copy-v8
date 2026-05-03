mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use gpt_copy_v8_backend::openrouter::{MockOpenRouterClient, OpenRouterError};
use serde_json::json;
use tokio::time::{Duration, sleep};
use tower::ServiceExt;
use uuid::Uuid;

fn post_json(uri: &str, value: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(common::json_body(value))
        .unwrap()
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

async fn create_conversation(app: &axum::Router, title: &str) -> String {
    let res = app
        .clone()
        .oneshot(post_json("/api/conversations", json!({ "title": title })))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = common::read_json(res).await;
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn send_rejects_empty_content() {
    let app = common::test_app_with_mock(MockOpenRouterClient::with_chat("hi back")).await;
    let conv_id = create_conversation(&app, "c").await;
    let res = app
        .oneshot(post_json(
            &format!("/api/conversations/{conv_id}/messages"),
            json!({ "content": "" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = common::read_json(res).await;
    assert_eq!(body["error"]["code"], "VALIDATION");
}

#[tokio::test]
async fn send_rejects_too_long_content() {
    let app = common::test_app_with_mock(MockOpenRouterClient::with_chat("hi back")).await;
    let conv_id = create_conversation(&app, "c").await;
    let too_long: String = "a".repeat(32_001);
    let res = app
        .oneshot(post_json(
            &format!("/api/conversations/{conv_id}/messages"),
            json!({ "content": too_long }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = common::read_json(res).await;
    assert_eq!(body["error"]["code"], "VALIDATION");
}

#[tokio::test]
async fn send_rejects_oversized_byte_content() {
    let app = common::test_app_with_mock(MockOpenRouterClient::with_chat("hi back")).await;
    let conv_id = create_conversation(&app, "c").await;
    let too_large = "🙂".repeat(16_001);
    let res = app
        .oneshot(post_json(
            &format!("/api/conversations/{conv_id}/messages"),
            json!({ "content": too_large }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = common::read_json(res).await;
    assert_eq!(body["error"]["code"], "VALIDATION");
}

#[tokio::test]
async fn send_returns_user_and_assistant() {
    let app = common::test_app_with_mock(MockOpenRouterClient::with_chat("Hello back")).await;
    let conv_id = create_conversation(&app, "c").await;
    let res = app
        .oneshot(post_json(
            &format!("/api/conversations/{conv_id}/messages"),
            json!({ "content": "Hello" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = common::read_json(res).await;
    assert_eq!(body["user_message"]["role"], "user");
    assert_eq!(body["user_message"]["content"], "Hello");
    assert_eq!(body["assistant_message"]["role"], "assistant");
    assert_eq!(body["assistant_message"]["content"], "Hello back");
    assert_eq!(body["user_message"]["conversation_id"], conv_id);
    assert_eq!(body["assistant_message"]["conversation_id"], conv_id);
}

#[tokio::test]
async fn send_404_when_conv_missing() {
    let app = common::test_app_with_mock(MockOpenRouterClient::with_chat("ok")).await;
    let bogus = Uuid::new_v4();
    let res = app
        .oneshot(post_json(
            &format!("/api/conversations/{bogus}/messages"),
            json!({ "content": "hi" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body = common::read_json(res).await;
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn send_502_when_upstream_fails() {
    let app = common::test_app_with_mock(MockOpenRouterClient::with_chat_error(
        OpenRouterError::HttpStatus(429),
    ))
    .await;
    let conv_id = create_conversation(&app, "c").await;
    let res = app
        .clone()
        .oneshot(post_json(
            &format!("/api/conversations/{conv_id}/messages"),
            json!({ "content": "hi" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_GATEWAY);
    let body = common::read_json(res).await;
    assert_eq!(body["error"]["code"], "UPSTREAM");

    let res = app
        .oneshot(get(&format!("/api/conversations/{conv_id}/messages")))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = common::read_json(res).await;
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_messages_returns_in_order() {
    let app = common::test_app_with_mock(MockOpenRouterClient::with_chat("a1")).await;
    let conv_id = create_conversation(&app, "c").await;

    // Send first message — uses initial mock.
    let res = app
        .clone()
        .oneshot(post_json(
            &format!("/api/conversations/{conv_id}/messages"),
            json!({ "content": "first" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // List — expect 2 (user + assistant).
    let res = app
        .oneshot(get(&format!("/api/conversations/{conv_id}/messages")))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = common::read_json(res).await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["role"], "user");
    assert_eq!(arr[0]["content"], "first");
    assert_eq!(arr[1]["role"], "assistant");
    assert_eq!(arr[1]["content"], "a1");
}

#[tokio::test]
async fn list_messages_404_when_conv_missing() {
    let app = common::test_app().await;
    let bogus = Uuid::new_v4();
    let res = app
        .oneshot(get(&format!("/api/conversations/{bogus}/messages")))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body = common::read_json(res).await;
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn stream_emits_tokens_then_done() {
    let app =
        common::test_app_with_mock(MockOpenRouterClient::with_stream(vec!["Hel", "lo"])).await;
    let conv_id = create_conversation(&app, "c").await;
    let res = app
        .oneshot(post_json(
            &format!("/api/conversations/{conv_id}/stream"),
            json!({ "content": "hi" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_text = common::read_text(res).await;

    // Parse SSE events.
    let mut tokens: Vec<String> = Vec::new();
    let mut saw_done = false;
    for block in body_text.split("\n\n") {
        let mut event_name: Option<String> = None;
        let mut data_line: Option<String> = None;
        for line in block.lines() {
            let line = line.trim_end_matches('\r');
            if let Some(rest) = line.strip_prefix("event:") {
                event_name = Some(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("data:") {
                data_line = Some(rest.trim_start().to_string());
            }
        }
        match (event_name.as_deref(), data_line) {
            (Some("token"), Some(d)) => tokens.push(d),
            (Some("done"), Some(d)) => {
                saw_done = true;
                let v: serde_json::Value = serde_json::from_str(&d).unwrap();
                assert!(v["message_id"].is_string());
                let id = v["message_id"].as_str().unwrap();
                assert!(Uuid::parse_str(id).is_ok());
            }
            _ => {}
        }
    }
    assert_eq!(tokens, vec!["Hel".to_string(), "lo".to_string()]);
    assert!(saw_done, "expected done event");
}

#[tokio::test]
async fn stream_empty_response_persists_user_and_emits_error() {
    let app = common::test_app_with_mock(MockOpenRouterClient::with_stream(vec![])).await;
    let conv_id = create_conversation(&app, "c").await;
    let res = app
        .clone()
        .oneshot(post_json(
            &format!("/api/conversations/{conv_id}/stream"),
            json!({ "content": "hi" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_text = common::read_text(res).await;
    assert!(body_text.contains("event: error"));
    assert!(body_text.contains("Empty response"));

    let res = app
        .oneshot(get(&format!("/api/conversations/{conv_id}/messages")))
        .await
        .unwrap();
    let body = common::read_json(res).await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["role"], "user");
    assert_eq!(arr[0]["content"], "hi");
}

#[tokio::test]
async fn stream_client_disconnect_persists_user_and_partial_content() {
    let app =
        common::test_app_with_mock(MockOpenRouterClient::with_stream(vec!["Hel", "lo"])).await;
    let conv_id = create_conversation(&app, "c").await;

    let res = app
        .clone()
        .oneshot(post_json(
            &format!("/api/conversations/{conv_id}/stream"),
            json!({ "content": "hi" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    drop(res);

    let mut body = serde_json::Value::Null;
    for _ in 0..20 {
        sleep(Duration::from_millis(25)).await;
        let res = app
            .clone()
            .oneshot(get(&format!("/api/conversations/{conv_id}/messages")))
            .await
            .unwrap();
        body = common::read_json(res).await;
        if body.as_array().is_some_and(|arr| arr.len() >= 2) {
            break;
        }
    }

    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["role"], "user");
    assert_eq!(arr[0]["content"], "hi");
    assert_eq!(arr[1]["role"], "assistant");
    assert!(arr[1]["content"].as_str().unwrap().starts_with("Hel"));
}

#[tokio::test]
async fn stream_error_sanitizes_and_does_not_persist_messages() {
    let app = common::test_app_with_mock(MockOpenRouterClient::with_stream_error(
        OpenRouterError::HttpStatus(429),
    ))
    .await;
    let conv_id = create_conversation(&app, "c").await;
    let res = app
        .clone()
        .oneshot(post_json(
            &format!("/api/conversations/{conv_id}/stream"),
            json!({ "content": "hi" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_text = common::read_text(res).await;
    assert!(body_text.contains("event: error"));
    assert!(body_text.contains("Provider unavailable"));
    assert!(!body_text.contains("429"));

    let res = app
        .oneshot(get(&format!("/api/conversations/{conv_id}/messages")))
        .await
        .unwrap();
    let body = common::read_json(res).await;
    assert_eq!(body.as_array().unwrap().len(), 0);
}
