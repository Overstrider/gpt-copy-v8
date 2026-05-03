use futures_util::StreamExt;
use gpt_copy_v8_backend::openrouter::{
    ChatMessage, HttpOpenRouterClient, OpenRouterClient, OpenRouterError,
};

#[tokio::test]
async fn http_chat_success() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices":[{"message":{"content":"hi"}}]}"#)
        .create_async()
        .await;

    let client = HttpOpenRouterClient::with_base_url("test-key".to_string(), server.url()).unwrap();
    let res = client
        .chat(
            "test-model",
            vec![ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
        )
        .await
        .unwrap();
    assert_eq!(res, "hi");
}

#[tokio::test]
async fn http_chat_429_maps_to_http_status() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("POST", "/chat/completions")
        .with_status(429)
        .with_body("rate limited")
        .create_async()
        .await;

    let client = HttpOpenRouterClient::with_base_url("test-key".to_string(), server.url()).unwrap();
    let err = client
        .chat(
            "test-model",
            vec![ChatMessage {
                role: "user".to_string(),
                content: "hi".to_string(),
            }],
        )
        .await
        .expect_err("expected error");
    match err {
        OpenRouterError::HttpStatus(429) => {}
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn http_chat_decode_error_when_no_choices() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices":[]}"#)
        .create_async()
        .await;

    let client = HttpOpenRouterClient::with_base_url("test-key".to_string(), server.url()).unwrap();
    let err = client
        .chat(
            "m",
            vec![ChatMessage {
                role: "user".to_string(),
                content: "x".to_string(),
            }],
        )
        .await
        .expect_err("expected error");
    match err {
        OpenRouterError::Decode(_) => {}
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn http_stream_parses_data_lines() {
    let mut server = mockito::Server::new_async().await;
    let body = "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\ndata: [DONE]\n\n";
    let _m = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = HttpOpenRouterClient::with_base_url("test-key".to_string(), server.url()).unwrap();
    let mut s = client
        .stream(
            "m",
            vec![ChatMessage {
                role: "user".to_string(),
                content: "x".to_string(),
            }],
        )
        .await
        .unwrap();
    let mut out = Vec::new();
    while let Some(item) = s.next().await {
        match item {
            Ok(v) => out.push(v),
            Err(e) => panic!("stream error: {e:?}"),
        }
    }
    assert_eq!(out, vec!["Hel".to_string(), "lo".to_string()]);
}

#[tokio::test]
async fn http_stream_skips_empty_delta() {
    let mut server = mockito::Server::new_async().await;
    let body = "data: {\"choices\":[{\"delta\":{}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"x\"}}]}\n\ndata: [DONE]\n\n";
    let _m = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = HttpOpenRouterClient::with_base_url("test-key".to_string(), server.url()).unwrap();
    let mut s = client
        .stream(
            "m",
            vec![ChatMessage {
                role: "user".to_string(),
                content: "y".to_string(),
            }],
        )
        .await
        .unwrap();
    let mut out = Vec::new();
    while let Some(item) = s.next().await {
        out.push(item.unwrap());
    }
    assert_eq!(out, vec!["x".to_string()]);
}

#[tokio::test]
async fn http_stream_terminates_on_done() {
    let mut server = mockito::Server::new_async().await;
    // Trailing data after [DONE] must be ignored.
    let body = "data: {\"choices\":[{\"delta\":{\"content\":\"a\"}}]}\n\ndata: [DONE]\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"ignored\"}}]}\n\n";
    let _m = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = HttpOpenRouterClient::with_base_url("test-key".to_string(), server.url()).unwrap();
    let mut s = client
        .stream(
            "m",
            vec![ChatMessage {
                role: "user".to_string(),
                content: "z".to_string(),
            }],
        )
        .await
        .unwrap();
    let mut out = Vec::new();
    while let Some(item) = s.next().await {
        out.push(item.unwrap());
    }
    assert_eq!(out, vec!["a".to_string()]);
}
