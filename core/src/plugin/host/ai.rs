//! AI Host Interface for Plugin System
//!
//! This module provides the host-side interface for AI capabilities to plugins.
//! It acts as a bridge between the plugin system and the core AI service,
//! handling permission checks and data conversion.

use crate::ai::AiService;
use crate::dto::openai::ChatCompletionRequest;
use crate::plugin::rustpress::plugin::ai::*;
use std::sync::Arc;

/// AI Helper - Provides secure AI capabilities to authorized plugins through the host interface
pub struct AiHelper {
    ai_service: Arc<AiService>,
}

impl AiHelper {
    /// Create a new AI helper instance
    pub fn new(ai_service: Arc<AiService>) -> Self {
        Self { ai_service }
    }
}

#[async_trait::async_trait]
impl Host for super::super::PluginHostState {
    async fn chat_completion(
        &mut self,
        request: crate::plugin::rustpress::plugin::ai::ChatCompletionRequest,
    ) -> Result<
        Result<crate::plugin::rustpress::plugin::ai::ChatCompletionResponse, String>,
        wasmtime::Error,
    > {
        // Check if plugin has AI chat permission
        if !self.granted_permissions.contains("ai:chat") {
            return Ok(Err(format!(
                "Plugin '{}' does not have 'ai:chat' permission",
                self.plugin_id
            )));
        }

        // Check if AI helper is available
        match &self.ai_helper {
            Some(ai_helper) => {
                // Convert WIT request to internal DTO
                let internal_request = crate::dto::openai::ChatCompletionRequest {
                    model: request.model,
                    messages: request
                        .messages
                        .into_iter()
                        .map(|msg| crate::dto::openai::ChatMessage {
                            role: msg.role,
                            content: msg.content,
                        })
                        .collect(),
                    max_tokens: request.max_tokens,
                };

                // Call AI service through helper
                match ai_helper
                    .ai_service
                    .chat_completion(&self.plugin_id, internal_request)
                    .await
                {
                    Ok(response) => {
                        // Convert response back to WIT format
                        let wit_response =
                            crate::plugin::rustpress::plugin::ai::ChatCompletionResponse {
                                id: response.id,
                                object: response.object,
                                created: response.created.try_into().unwrap_or(0),
                                model: response.model,
                                choices: response
                                    .choices
                                    .into_iter()
                                    .map(|choice| {
                                        crate::plugin::rustpress::plugin::ai::ChatCompletionChoice {
                                            message:
                                                crate::plugin::rustpress::plugin::ai::ChatMessage {
                                                    role: choice.message.role,
                                                    content: choice.message.content,
                                                },
                                            finish_reason: choice
                                                .finish_reason
                                                .unwrap_or_else(|| "unknown".to_string()),
                                        }
                                    })
                                    .collect(),
                                usage: crate::plugin::rustpress::plugin::ai::ChatCompletionUsage {
                                    prompt_tokens: response
                                        .usage
                                        .prompt_tokens
                                        .try_into()
                                        .unwrap_or(0),
                                    completion_tokens: response
                                        .usage
                                        .completion_tokens
                                        .try_into()
                                        .unwrap_or(0),
                                    total_tokens: response
                                        .usage
                                        .total_tokens
                                        .try_into()
                                        .unwrap_or(0),
                                },
                            };
                        Ok(Ok(wit_response))
                    }
                    Err(e) => Ok(Err(e)),
                }
            }
            None => Ok(Err("AI functionality is not available".to_string())),
        }
    }

    async fn list_models(&mut self) -> Result<Vec<String>, wasmtime::Error> {
        // Check if plugin has AI list_models permission
        if !self.granted_permissions.contains("ai:list_models") {
            // For non-result functions, we need to return an error in a different way
            // Since this is a host function, we can panic or return an empty list
            // For now, return empty list as a way to indicate permission denied
            return Ok(vec![]);
        }

        // Check if AI helper is available
        match &self.ai_helper {
            Some(ai_helper) => {
                // Call AI service through helper
                match ai_helper.ai_service.list_models(&self.plugin_id).await {
                    Ok(response) => {
                        let model_ids = response
                            .models
                            .into_iter()
                            .map(|model| model.id)
                            .collect::<Vec<String>>();
                        Ok(model_ids)
                    }
                    Err(_) => Ok(vec![]), // Return empty list on error
                }
            }
            None => Ok(vec![]), // Return empty list if AI not available
        }
    }
}
