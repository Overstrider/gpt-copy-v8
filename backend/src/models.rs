use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "user" => Some(Self::User),
            "assistant" => Some(Self::Assistant),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Conversation {
    pub id: uuid::Uuid,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: uuid::Uuid,
    pub conversation_id: uuid::Uuid,
    pub role: Role,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateConversationReq {
    pub title: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateMessageReq {
    pub content: String,
}

#[derive(Debug, serde::Serialize)]
pub struct SendMessageRes {
    pub user_message: Message,
    pub assistant_message: Message,
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

pub fn validate_title(t: &str) -> Result<(), AppError> {
    let trimmed = t.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("title must not be empty".to_string()));
    }
    if trimmed.chars().count() > 200 {
        return Err(AppError::Validation(
            "title must be at most 200 characters".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_content(c: &str) -> Result<(), AppError> {
    if c.trim().is_empty() {
        return Err(AppError::Validation(
            "content must not be empty".to_string(),
        ));
    }
    if c.chars().count() > 32_000 {
        return Err(AppError::Validation(
            "content must be at most 32000 characters".to_string(),
        ));
    }
    Ok(())
}
