mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::json;
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

#[tokio::test]
async fn list_returns_empty_array() {
    let app = common::test_app().await;
    let res = app.oneshot(get("/api/conversations")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = common::read_json(res).await;
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn create_rejects_empty_title() {
    let app = common::test_app().await;
    let res = app
        .oneshot(post_json("/api/conversations", json!({ "title": "" })))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = common::read_json(res).await;
    assert_eq!(body["error"]["code"], "VALIDATION");
}

#[tokio::test]
async fn create_rejects_too_long_title() {
    let app = common::test_app().await;
    let too_long: String = "a".repeat(201);
    let res = app
        .oneshot(post_json(
            "/api/conversations",
            json!({ "title": too_long }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = common::read_json(res).await;
    assert_eq!(body["error"]["code"], "VALIDATION");
}

#[tokio::test]
async fn create_persists_and_list_returns() {
    let app = common::test_app().await;

    let res = app
        .clone()
        .oneshot(post_json(
            "/api/conversations",
            json!({ "title": "First chat" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = common::read_json(res).await;
    assert_eq!(body["title"], "First chat");
    let id_str = body["id"].as_str().unwrap();
    assert!(Uuid::parse_str(id_str).is_ok());

    let res = app.oneshot(get("/api/conversations")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = common::read_json(res).await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["title"], "First chat");
    assert_eq!(arr[0]["id"], id_str);
}

#[tokio::test]
async fn create_returns_uuid_v4() {
    let app = common::test_app().await;
    let res = app
        .oneshot(post_json("/api/conversations", json!({ "title": "Hi" })))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = common::read_json(res).await;
    let id_str = body["id"].as_str().unwrap();
    let parsed = Uuid::parse_str(id_str).unwrap();
    assert_eq!(parsed.get_version_num(), 4);
}
