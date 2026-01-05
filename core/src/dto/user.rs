use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::entity::users;

/// User role enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
pub enum UserRole {
    Root,
    Admin,
    User,
}

/// User business entity (DTO)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub salt: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request DTO for creating a user
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

/// Login request DTO
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response DTO
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Refresh token request DTO
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Refresh token response DTO
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// User response DTO (does not contain sensitive information)
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Convert from User DTO to UserResponse
impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Convert from database Model to business DTO
impl From<users::Model> for User {
    fn from(model: users::Model) -> Self {
        let role = match model.role.as_str() {
            "Root" => UserRole::Root,
            "Admin" => UserRole::Admin,
            _ => UserRole::User,
        };
        
        User {
            id: model.id,
            username: model.username,
            email: model.email,
            role,
            salt: model.salt,
            password_hash: model.password_hash,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        }
    }
}

