use crate::dto::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, CreateOpenAIApiKeyRequest,
    ListOpenAIModelsResponse, OpenAIApiKeyResponse, OpenAIModel, SetDefaultModelRequest,
    TestOpenAIApiKeyResponse, TestTokenUsage, UpdateOpenAIApiKeyRequest,
};
use crate::entity::openai_api_keys;
use crate::repository::{PostRepository, UserRepository};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::Value as JsonValue;
use std::sync::Arc;

// Helper function to get database connection from state
fn get_db_connection<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    state: &Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>,
) -> Arc<sea_orm::DatabaseConnection> {
    state.db.clone()
}

// Helper function to mask API key
fn mask_api_key(key: &str) -> String {
    if key.len() > 7 {
        format!("{}...", &key[..7])
    } else {
        "***".to_string()
    }
}

/// Get all OpenAI API keys
#[utoipa::path(
    get,
    path = "/api/admin/openai/keys",
    responses(
        (status = 200, description = "Successfully retrieved API keys", body = Vec<OpenAIApiKeyResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn get_openai_keys<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<Json<Vec<OpenAIApiKeyResponse>>, StatusCode> {
    let db = get_db_connection(&state);

    let keys = openai_api_keys::Entity::find()
        .all(db.as_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch OpenAI API keys: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let result: Vec<OpenAIApiKeyResponse> = keys
        .into_iter()
        .map(|k| OpenAIApiKeyResponse {
            id: k.id,
            name: k.name,
            api_key_masked: mask_api_key(&k.api_key),
            endpoint: k.endpoint,
            is_default: k.is_default,
            default_model: k.default_model,
            created_at: k.created_at.into(),
            updated_at: k.updated_at.into(),
        })
        .collect();

    Ok(Json(result))
}

/// Create a new OpenAI API key
#[utoipa::path(
    post,
    path = "/api/admin/openai/keys",
    request_body = CreateOpenAIApiKeyRequest,
    responses(
        (status = 200, description = "Successfully created API key", body = OpenAIApiKeyResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn create_openai_key<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::Json(payload): axum::Json<CreateOpenAIApiKeyRequest>,
) -> Result<Json<OpenAIApiKeyResponse>, StatusCode> {
    let db = get_db_connection(&state);

    // If this is set as default, unset other defaults
    if payload.is_default.unwrap_or(false) {
        let all_keys = openai_api_keys::Entity::find()
            .all(db.as_ref())
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch OpenAI API keys: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for key in all_keys {
            let mut active_model: openai_api_keys::ActiveModel = key.into();
            active_model.is_default = Set(false);
            active_model.update(db.as_ref()).await.map_err(|e| {
                tracing::error!("Failed to update OpenAI API key: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    let endpoint = payload
        .endpoint
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

    let new_key = openai_api_keys::ActiveModel {
        name: Set(payload.name),
        api_key: Set(payload.api_key),
        endpoint: Set(Some(endpoint)),
        is_default: Set(payload.is_default.unwrap_or(false)),
        default_model: Set(payload.default_model),
        ..Default::default()
    };

    let key = new_key.insert(db.as_ref()).await.map_err(|e| {
        tracing::error!("Failed to create OpenAI API key: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(OpenAIApiKeyResponse {
        id: key.id,
        name: key.name,
        api_key_masked: mask_api_key(&key.api_key),
        endpoint: key.endpoint,
        is_default: key.is_default,
        default_model: key.default_model,
        created_at: key.created_at.into(),
        updated_at: key.updated_at.into(),
    }))
}

/// Update an OpenAI API key
#[utoipa::path(
    put,
    path = "/api/admin/openai/keys/{id}",
    params(
        ("id" = i64, Path, description = "API key ID")
    ),
    request_body = UpdateOpenAIApiKeyRequest,
    responses(
        (status = 200, description = "Successfully updated API key", body = OpenAIApiKeyResponse),
        (status = 404, description = "API key not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn update_openai_key<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path(id): Path<i64>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::Json(payload): axum::Json<UpdateOpenAIApiKeyRequest>,
) -> Result<Json<OpenAIApiKeyResponse>, StatusCode> {
    let db = get_db_connection(&state);

    let key = openai_api_keys::Entity::find_by_id(id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch OpenAI API key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // If setting as default, unset other defaults
    if payload.is_default == Some(true) {
        let all_keys = openai_api_keys::Entity::find()
            .filter(openai_api_keys::Column::Id.ne(id))
            .all(db.as_ref())
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch OpenAI API keys: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for other_key in all_keys {
            let mut active_model: openai_api_keys::ActiveModel = other_key.into();
            active_model.is_default = Set(false);
            active_model.update(db.as_ref()).await.map_err(|e| {
                tracing::error!("Failed to update OpenAI API key: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    let mut active_model: openai_api_keys::ActiveModel = key.into();

    if let Some(name) = payload.name {
        active_model.name = Set(name);
    }
    if let Some(api_key) = payload.api_key {
        active_model.api_key = Set(api_key);
    }
    if let Some(endpoint) = payload.endpoint {
        if endpoint.is_empty() {
            active_model.endpoint = Set(Some("https://api.openai.com/v1".to_string()));
        } else {
            active_model.endpoint = Set(Some(endpoint));
        }
    }
    if let Some(is_default) = payload.is_default {
        active_model.is_default = Set(is_default);
    }
    if let Some(default_model) = payload.default_model {
        if default_model.is_empty() {
            // Empty string means set to null (use API default)
            active_model.default_model = Set(None);
        } else {
            // Non-empty string means set to specific model
            active_model.default_model = Set(Some(default_model));
        }
    }

    let updated_key = active_model.update(db.as_ref()).await.map_err(|e| {
        tracing::error!("Failed to update OpenAI API key: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(OpenAIApiKeyResponse {
        id: updated_key.id,
        name: updated_key.name,
        api_key_masked: mask_api_key(&updated_key.api_key),
        endpoint: updated_key.endpoint,
        is_default: updated_key.is_default,
        default_model: updated_key.default_model,
        created_at: updated_key.created_at.into(),
        updated_at: updated_key.updated_at.into(),
    }))
}

/// Delete an OpenAI API key
#[utoipa::path(
    delete,
    path = "/api/admin/openai/keys/{id}",
    params(
        ("id" = i64, Path, description = "API key ID")
    ),
    responses(
        (status = 200, description = "Successfully deleted API key"),
        (status = 404, description = "API key not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn delete_openai_key<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path(id): Path<i64>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<StatusCode, StatusCode> {
    let db = get_db_connection(&state);

    let key = openai_api_keys::Entity::find_by_id(id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch OpenAI API key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if key.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    openai_api_keys::Entity::delete_by_id(id)
        .exec(db.as_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete OpenAI API key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

/// Test an OpenAI API key
#[utoipa::path(
    post,
    path = "/api/admin/openai/keys/{id}/test",
    params(
        ("id" = i64, Path, description = "API key ID")
    ),
    responses(
        (status = 200, description = "API key test result", body = TestOpenAIApiKeyResponse),
        (status = 404, description = "API key not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn test_openai_key<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path(id): Path<i64>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<Json<TestOpenAIApiKeyResponse>, StatusCode> {
    let db = get_db_connection(&state);

    let key = openai_api_keys::Entity::find_by_id(id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch OpenAI API key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if key.api_key.is_empty() {
        return Ok(Json(TestOpenAIApiKeyResponse {
            success: false,
            message: "API key is empty".to_string(),
            response_content: None,
            model_used: None,
            token_usage: None,
        }));
    }

    // Test the API key by making a chat completion request
    let endpoint = key
        .endpoint
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");

    // Create a chat completion request
    // Only include model if default_model is configured, otherwise let API use its default
    let chat_request = ChatCompletionRequest {
        model: key.default_model.clone(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Output \"Hello World\" in both Chinese and English, and also output the model used.".to_string(),
        }],
        max_tokens: Some(100),
    };

    let client = reqwest::Client::new();
    let chat_url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));

    let response = client
        .post(&chat_url)
        .header("Authorization", format!("Bearer {}", key.api_key))
        .header("Content-Type", "application/json")
        .json(&chat_request)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<ChatCompletionResponse>().await {
                    Ok(chat_response) => {
                        let response_content = chat_response
                            .choices
                            .first()
                            .map(|choice| choice.message.content.clone())
                            .unwrap_or("No response content".to_string());

                        let token_usage = Some(TestTokenUsage {
                            prompt_tokens: chat_response.usage.prompt_tokens,
                            completion_tokens: chat_response.usage.completion_tokens,
                            total_tokens: chat_response.usage.total_tokens,
                        });

                        Ok(Json(TestOpenAIApiKeyResponse {
                            success: true,
                            message: "API key is valid and chat completion works".to_string(),
                            response_content: Some(response_content),
                            model_used: Some(chat_response.model),
                            token_usage,
                        }))
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse chat response: {}", e);
                        Ok(Json(TestOpenAIApiKeyResponse {
                            success: true,
                            message: "API key is valid but failed to parse response".to_string(),
                            response_content: None,
                            model_used: None,
                            token_usage: None,
                        }))
                    }
                }
            } else {
                let status = resp.status();
                let error_text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Ok(Json(TestOpenAIApiKeyResponse {
                    success: false,
                    message: format!("API returned error: {} - {}", status, error_text),
                    response_content: None,
                    model_used: None,
                    token_usage: None,
                }))
            }
        }
        Err(e) => {
            tracing::error!("Failed to test API key: {}", e);
            Ok(Json(TestOpenAIApiKeyResponse {
                success: false,
                message: format!("Failed to connect to API: {}", e),
                response_content: None,
                model_used: None,
                token_usage: None,
            }))
        }
    }
}

/// List available OpenAI models for a specific API key
#[utoipa::path(
    get,
    path = "/api/admin/openai/keys/{id}/models",
    params(
        ("id" = i64, Path, description = "API key ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved models", body = ListOpenAIModelsResponse),
        (status = 404, description = "API key not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn list_openai_models<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path(id): Path<i64>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<Json<ListOpenAIModelsResponse>, StatusCode> {
    let db = get_db_connection(&state);

    let key = openai_api_keys::Entity::find_by_id(id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch OpenAI API key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Fetch models from the configured endpoint
    let endpoint = key
        .endpoint
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");
    let models_url = format!("{}/models", endpoint.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let response = client
        .get(&models_url)
        .header("Authorization", format!("Bearer {}", key.api_key))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch models from API: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Request: {:?}", models_url);
    tracing::info!("Response: {:?}", response);

    if !response.status().is_success() {
        let status = response.status();
        let _error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
        );
    }

    let models_data: JsonValue = response.json().await.map_err(|e| {
        tracing::error!("Failed to parse models response: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Check if this is a Google Vertex AI response or OpenAI response
    let models_array = if models_data.get("data").is_some() {
        // OpenAI format or Google Vertex AI format with "data" field
        models_data
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
    } else if models_data.get("models").is_some() {
        // Google AI API format (standard Gemini REST API)
        models_data
            .get("models")
            .and_then(|d| d.as_array())
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
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
                Utc::now().timestamp()
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

    Ok(Json(ListOpenAIModelsResponse {
        models,
        default_model: key.default_model.clone(),
    }))
}

/// Set default model for an API key
#[utoipa::path(
    post,
    path = "/api/admin/openai/keys/{id}/models",
    params(
        ("id" = i64, Path, description = "API key ID")
    ),
    request_body = SetDefaultModelRequest,
    responses(
        (status = 200, description = "Successfully set default model", body = OpenAIApiKeyResponse),
        (status = 404, description = "API key not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn set_default_model<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path(id): Path<i64>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::Json(payload): axum::Json<SetDefaultModelRequest>,
) -> Result<Json<OpenAIApiKeyResponse>, StatusCode> {
    let db = get_db_connection(&state);

    let key = openai_api_keys::Entity::find_by_id(id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch OpenAI API key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active_model: openai_api_keys::ActiveModel = key.into();
    active_model.default_model = Set(Some(payload.model_id));
    let updated_key = active_model.update(db.as_ref()).await.map_err(|e| {
        tracing::error!("Failed to update OpenAI API key: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(OpenAIApiKeyResponse {
        id: updated_key.id,
        name: updated_key.name,
        api_key_masked: mask_api_key(&updated_key.api_key),
        endpoint: updated_key.endpoint,
        is_default: updated_key.is_default,
        default_model: updated_key.default_model,
        created_at: updated_key.created_at.into(),
        updated_at: updated_key.updated_at.into(),
    }))
}
