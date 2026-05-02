use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{StreamExt, stream::BoxStream};

#[derive(Debug, thiserror::Error)]
pub enum OpenRouterError {
    #[error("transport: {0}")]
    Transport(String),
    #[error("upstream status: {0}")]
    HttpStatus(u16),
    #[error("decode: {0}")]
    Decode(String),
    #[error("stream: {0}")]
    Stream(String),
}

impl From<reqwest::Error> for OpenRouterError {
    fn from(e: reqwest::Error) -> Self {
        OpenRouterError::Transport(e.to_string())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Serialize)]
struct ChatCompletionReq<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
}

#[derive(Debug, serde::Deserialize)]
struct ChatCompletionRes {
    choices: Vec<ChoiceFull>,
}

#[derive(Debug, serde::Deserialize)]
struct ChoiceFull {
    message: AssistantMsg,
}

#[derive(Debug, serde::Deserialize)]
struct AssistantMsg {
    content: String,
}

#[derive(Debug, serde::Deserialize)]
struct StreamChunk {
    choices: Vec<ChoiceDelta>,
}

#[derive(Debug, serde::Deserialize)]
struct ChoiceDelta {
    #[serde(default)]
    delta: Delta,
}

#[derive(Debug, Default, serde::Deserialize)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
}

#[async_trait]
pub trait OpenRouterClient: Send + Sync {
    async fn chat(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<String, OpenRouterError>;

    async fn stream(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<BoxStream<'static, Result<String, OpenRouterError>>, OpenRouterError>;
}

pub struct HttpOpenRouterClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl HttpOpenRouterClient {
    pub fn new(api_key: String) -> Result<Self, OpenRouterError> {
        Self::with_base_url(api_key, "https://openrouter.ai/api/v1".to_string())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, OpenRouterError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            http,
            api_key,
            base_url,
        })
    }
}

#[async_trait]
impl OpenRouterClient for HttpOpenRouterClient {
    async fn chat(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<String, OpenRouterError> {
        let body = ChatCompletionReq {
            model,
            messages: &messages,
            stream: false,
        };
        let url = format!("{}/chat/completions", self.base_url);
        let res = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", "http://localhost")
            .header("X-Title", "gpt-copy-v8")
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            return Err(OpenRouterError::HttpStatus(status.as_u16()));
        }

        let parsed: ChatCompletionRes = res
            .json()
            .await
            .map_err(|e| OpenRouterError::Decode(e.to_string()))?;

        let content = parsed
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| OpenRouterError::Decode("missing choices".to_string()))?
            .message
            .content;

        Ok(content)
    }

    async fn stream(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<BoxStream<'static, Result<String, OpenRouterError>>, OpenRouterError> {
        let body = ChatCompletionReq {
            model,
            messages: &messages,
            stream: true,
        };
        let url = format!("{}/chat/completions", self.base_url);
        let res = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", "http://localhost")
            .header("X-Title", "gpt-copy-v8")
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            return Err(OpenRouterError::HttpStatus(status.as_u16()));
        }

        let byte_stream = res.bytes_stream();
        let parsed = parse_sse_stream(byte_stream);
        Ok(Box::pin(parsed))
    }
}

fn extract_delta(chunk: StreamChunk) -> Option<String> {
    let choice = chunk.choices.into_iter().next()?;
    let content = choice.delta.content?;
    if content.is_empty() {
        None
    } else {
        Some(content)
    }
}

fn parse_sse_stream<S>(byte_stream: S) -> BoxStream<'static, Result<String, OpenRouterError>>
where
    S: futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
{
    async_stream_helper(byte_stream)
}

fn async_stream_helper<S>(byte_stream: S) -> BoxStream<'static, Result<String, OpenRouterError>>
where
    S: futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
{
    use futures_util::stream::unfold;

    struct State {
        inner: BoxStream<'static, Result<bytes::Bytes, reqwest::Error>>,
        buf: String,
        done: bool,
        pending: std::collections::VecDeque<Result<String, OpenRouterError>>,
    }

    let initial = State {
        inner: Box::pin(byte_stream),
        buf: String::new(),
        done: false,
        pending: std::collections::VecDeque::new(),
    };

    let stream = unfold(initial, |mut s: State| async move {
        loop {
            if let Some(item) = s.pending.pop_front() {
                return Some((item, s));
            }
            if s.done {
                return None;
            }
            match s.inner.next().await {
                Some(Ok(chunk)) => {
                    let text = match std::str::from_utf8(&chunk) {
                        Ok(t) => t.to_string(),
                        Err(e) => {
                            s.done = true;
                            return Some((Err(OpenRouterError::Stream(e.to_string())), s));
                        }
                    };
                    s.buf.push_str(&text);
                    // Process complete events terminated by \n\n
                    while let Some(idx) = s.buf.find("\n\n") {
                        let event_text = s.buf[..idx].to_string();
                        s.buf.drain(..idx + 2);
                        for line in event_text.lines() {
                            let line = line.trim_end_matches('\r');
                            if let Some(rest) = line.strip_prefix("data:") {
                                let payload = rest.trim_start();
                                if payload == "[DONE]" {
                                    s.done = true;
                                    if let Some(item) = s.pending.pop_front() {
                                        return Some((item, s));
                                    }
                                    return None;
                                }
                                match serde_json::from_str::<StreamChunk>(payload) {
                                    Ok(chunk) => {
                                        if let Some(content) = extract_delta(chunk) {
                                            s.pending.push_back(Ok(content));
                                        }
                                    }
                                    Err(e) => {
                                        s.pending
                                            .push_back(Err(OpenRouterError::Stream(e.to_string())));
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    s.done = true;
                    return Some((Err(OpenRouterError::Transport(e.to_string())), s));
                }
                None => {
                    s.done = true;
                    // Process any final buffered event without trailing \n\n
                    if !s.buf.trim().is_empty() {
                        let event_text = std::mem::take(&mut s.buf);
                        for line in event_text.lines() {
                            let line = line.trim_end_matches('\r');
                            if let Some(rest) = line.strip_prefix("data:") {
                                let payload = rest.trim_start();
                                if payload == "[DONE]" {
                                    if let Some(item) = s.pending.pop_front() {
                                        return Some((item, s));
                                    }
                                    return None;
                                }
                                if let Ok(chunk) = serde_json::from_str::<StreamChunk>(payload)
                                    && let Some(content) = extract_delta(chunk)
                                {
                                    s.pending.push_back(Ok(content));
                                }
                            }
                        }
                    }
                    if let Some(item) = s.pending.pop_front() {
                        return Some((item, s));
                    }
                    return None;
                }
            }
        }
    });

    Box::pin(stream)
}

pub struct MockOpenRouterClient {
    chat_response: Mutex<Option<Result<String, OpenRouterError>>>,
    stream_chunks: Mutex<Option<Vec<Result<String, OpenRouterError>>>>,
}

impl MockOpenRouterClient {
    pub fn with_chat(text: impl Into<String>) -> Self {
        Self {
            chat_response: Mutex::new(Some(Ok(text.into()))),
            stream_chunks: Mutex::new(Some(Vec::new())),
        }
    }

    pub fn with_stream(chunks: Vec<&str>) -> Self {
        Self {
            chat_response: Mutex::new(Some(Ok(String::new()))),
            stream_chunks: Mutex::new(Some(
                chunks.into_iter().map(|c| Ok(c.to_string())).collect(),
            )),
        }
    }

    pub fn with_chat_error(err: OpenRouterError) -> Self {
        Self {
            chat_response: Mutex::new(Some(Err(err))),
            stream_chunks: Mutex::new(Some(Vec::new())),
        }
    }

    pub fn with_stream_error(err: OpenRouterError) -> Self {
        Self {
            chat_response: Mutex::new(Some(Ok(String::new()))),
            stream_chunks: Mutex::new(Some(vec![Err(err)])),
        }
    }
}

#[async_trait]
impl OpenRouterClient for MockOpenRouterClient {
    async fn chat(
        &self,
        _model: &str,
        _messages: Vec<ChatMessage>,
    ) -> Result<String, OpenRouterError> {
        let mut guard = self.chat_response.lock().expect("mock chat poisoned");
        match guard.take() {
            Some(Ok(s)) => Ok(s),
            Some(Err(e)) => Err(e),
            None => Err(OpenRouterError::Decode("mock chat consumed".to_string())),
        }
    }

    async fn stream(
        &self,
        _model: &str,
        _messages: Vec<ChatMessage>,
    ) -> Result<BoxStream<'static, Result<String, OpenRouterError>>, OpenRouterError> {
        let mut guard = self.stream_chunks.lock().expect("mock stream poisoned");
        let chunks = guard.take().unwrap_or_default();
        let stream = futures_util::stream::iter(chunks);
        Ok(Box::pin(stream))
    }
}
