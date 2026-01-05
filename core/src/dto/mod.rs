//! Data Transfer Objects (DTOs)
//!
//! This module contains all DTOs used by the business layer for:
//! - API requests/responses
//! - Data transfer in the business logic layer
//! - Conversion with database Entities

pub mod admin;
pub mod openai;
pub mod plugin;
pub mod post;
pub mod user;

// Re-export commonly used DTOs
pub use admin::{
    AdminBanUserRequest, AdminPluginEnableResponse, AdminPluginListResponse, AdminPluginUpdateRequest, AdminPostListResponse,
    AdminResetPasswordRequest, AdminResetPasswordResponse,
    AdminUserListResponse,
};
pub use openai::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, CreateOpenAIApiKeyRequest,
    ListOpenAIModelsResponse, OpenAIApiKeyResponse, OpenAIModel, SetDefaultModelRequest,
    TestOpenAIApiKeyResponse, TestTokenUsage, UpdateOpenAIApiKeyRequest,
};
pub use plugin::{
    ApprovePluginPermissionsRequest,
    PluginInstallRequest, PluginPermissionsResponse, UpdatePluginPermissionsRequest,
};
pub use post::{
    CreatePostRequest, Post, PostDraft, PostDraftResponse, PostResponse, PostVersion,
    PostVersionResponse, SaveDraftRequest, UpdatePostRequest,
};
pub use user::{
    CreateUserRequest, LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse,
    User, UserResponse, UserRole,
};
