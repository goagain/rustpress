//! Data Transfer Objects (DTOs)
//! 
//! This module contains all DTOs used by the business layer for:
//! - API requests/responses
//! - Data transfer in the business logic layer
//! - Conversion with database Entities

pub mod post;
pub mod user;

// Re-export commonly used DTOs
pub use post::{
    Post, CreatePostRequest, UpdatePostRequest, PostResponse,
    PostVersion, PostVersionResponse, CreateVersionRequest,
    PostDraft, PostDraftResponse, SaveDraftRequest,
};
pub use user::{
    User, UserRole, CreateUserRequest, LoginRequest, LoginResponse,
    RefreshTokenRequest, RefreshTokenResponse, UserResponse,
};

