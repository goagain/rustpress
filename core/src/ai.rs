//! Core AI Module
//!
//! This module provides the core AI functionality used by various parts of the system,
//! including plugins, admin API, and other components. It abstracts the AI provider
//! interactions and provides a clean interface for AI operations.

use crate::dto::openai::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ListOpenAIModelsResponse,
    OpenAIModel,
};
use crate::repository::OpenAIApiKeyRepository;
use chrono;
use std::sync::Arc;

/// Core AI service that handles AI operations
pub struct AiService {
    repository: Arc<dyn OpenAIApiKeyRepository>,
}

impl AiService {
    /// Create a new AI service instance
    pub fn new(repository: Arc<dyn OpenAIApiKeyRepository>) -> Self {
        Self { repository }
    }

    /// Check if a plugin/client has permission to use AI capabilities
    pub async fn check_ai_permission(
        &self,
        plugin_id: &str,
        required_permission: &str,
    ) -> Result<bool, String> {
        // TODO: Implement permission checking via repository
        // For now, assume permission check is done at the host level
        Ok(true)
    }

    /// Get the default OpenAI API key for AI operations
    async fn get_default_api_key(&self) -> Result<crate::entity::openai_api_keys::Model, String> {
        let key = self
            .repository
            .find_default()
            .await
            .map_err(|e| format!("Failed to fetch default API key: {}", e))?
            .ok_or_else(|| "No default OpenAI API key configured".to_string())?;

        Ok(key)
    }

    /// Perform a chat completion using the default API key
    pub async fn chat_completion(
        &self,
        plugin_id: &str,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, String> {
        // Check permission
        if !self.check_ai_permission(plugin_id, "ai:chat").await? {
            return Err(format!(
                "Plugin '{}' does not have 'ai:chat' permission",
                plugin_id
            ));
        }

        let api_key = self.get_default_api_key().await?;

        // Prepare the HTTP request
        let endpoint = api_key
            .endpoint
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let chat_url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));

        let client = reqwest::Client::new();
        let response = client
            .post(&chat_url)
            .header("Authorization", format!("Bearer {}", api_key.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("API returned error: {} - {}", status, error_text));
        }

        let chat_response: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(chat_response)
    }

    /// List available models (for plugin discovery)
    pub async fn list_models(&self, plugin_id: &str) -> Result<ListOpenAIModelsResponse, String> {
        // Check permission
        if !self
            .check_ai_permission(plugin_id, "ai:list_models")
            .await?
        {
            return Err(format!(
                "Plugin '{}' does not have 'ai:list_models' permission",
                plugin_id
            ));
        }

        let api_key = self.get_default_api_key().await?;

        // Fetch models from the configured endpoint
        let endpoint = api_key
            .endpoint
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let models_url = format!("{}/models", endpoint.trim_end_matches('/'));

        let client = reqwest::Client::new();
        let response = client
            .get(&models_url)
            .header("Authorization", format!("Bearer {}", api_key.api_key))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch models: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("API returned error: {} - {}", status, error_text));
        }

        let models_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse models response: {}", e))?;

        // Check if this is a Google Vertex AI response or OpenAI response
        let models_array = if models_data.get("data").is_some() {
            // OpenAI format or Google Vertex AI format with "data" field
            models_data
                .get("data")
                .and_then(|d| d.as_array())
                .ok_or_else(|| "Invalid models response format".to_string())?
        } else if models_data.get("models").is_some() {
            // Google AI API format (standard Gemini REST API)
            models_data
                .get("models")
                .and_then(|d| d.as_array())
                .ok_or_else(|| "Invalid models response format".to_string())?
        } else {
            return Err("Invalid models response format".to_string());
        };

        let models: Vec<OpenAIModel> = models_array
            .iter()
            .filter_map(|m| {
                let id = m.get("id")?.as_str()?.to_string();
                let object = m.get("object")?.as_str()?.to_string();
                let owned_by = m.get("owned_by")?.as_str()?.to_string();

                // Handle different API formats
                let created = if let Some(created_val) = m.get("created") {
                    // OpenAI format
                    created_val.as_i64()?
                } else {
                    // Google Vertex AI format - use current timestamp as fallback
                    chrono::Utc::now().timestamp()
                };

                // Clean up Google Vertex AI model IDs (remove "models/" prefix)
                let clean_id = if id.starts_with("models/") {
                    id.strip_prefix("models/").unwrap_or(&id).to_string()
                } else {
                    id
                };

                Some(OpenAIModel {
                    id: clean_id,
                    object,
                    created,
                    owned_by,
                })
            })
            .collect();

        Ok(ListOpenAIModelsResponse {
            models,
            default_model: api_key.default_model,
        })
    }
}
