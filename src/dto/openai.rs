use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// OpenAI API key response (with masked key)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OpenAIApiKeyResponse {
    pub id: i64,
    pub name: String,
    pub api_key_masked: String,
    pub endpoint: Option<String>,
    pub is_default: bool,
    pub default_model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create OpenAI API key request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOpenAIApiKeyRequest {
    pub name: String,
    pub api_key: String,
    pub endpoint: Option<String>,
    pub is_default: Option<bool>,
    pub default_model: Option<String>,
}

/// Update OpenAI API key request
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOpenAIApiKeyRequest {
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub is_default: Option<bool>,
    pub default_model: Option<String>,
}

/// Test OpenAI API key response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TestOpenAIApiKeyResponse {
    pub success: bool,
    pub message: String,
    pub response_content: Option<String>,
    pub model_used: Option<String>,
    pub token_usage: Option<TestTokenUsage>,
}

/// Token usage information for test response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TestTokenUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

/// OpenAI model information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OpenAIModel {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

/// List OpenAI models response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListOpenAIModelsResponse {
    pub models: Vec<OpenAIModel>,
    pub default_model: Option<String>,
}

/// Set default model request
#[derive(Debug, Deserialize, ToSchema)]
pub struct SetDefaultModelRequest {
    pub model_id: String,
}

/// Chat completion message
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Chat completion request
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatCompletionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<i32>,
}

/// Chat completion choice
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatCompletionChoice {
    pub index: i32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// Chat completion usage
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatCompletionUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

/// Chat completion response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: ChatCompletionUsage,
}
