use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // user id
    pub username: String,
    pub role: String,
    pub exp: i64, // expiration time
    pub iat: i64, // issued at
    pub token_type: String, // "access" or "refresh"
}

impl Claims {
    pub fn new(user_id: i64, username: String, role: String, token_type: String) -> Self {
        let now = Utc::now();
        let exp = match token_type.as_str() {
            "access" => now + Duration::hours(1), // Access token expires in 1 hour
            "refresh" => now + Duration::days(7),  // Refresh token expires in 7 days
            _ => now + Duration::hours(1),
        };

        Claims {
            sub: user_id,
            username,
            role,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type,
        }
    }
}

/// JWT utility
pub struct JwtUtil;

impl JwtUtil {
    /// Get JWT secret key
    fn get_secret() -> String {
        env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string())
    }

    /// Generate Access Token
    pub fn generate_access_token(user_id: i64, username: String, role: String) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = Claims::new(user_id, username, role, "access".to_string());
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(Self::get_secret().as_ref()),
        )
    }

    /// Generate Refresh Token
    pub fn generate_refresh_token(user_id: i64, username: String, role: String) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = Claims::new(user_id, username, role, "refresh".to_string());
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(Self::get_secret().as_ref()),
        )
    }

    /// Verify and parse Token
    pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(Self::get_secret().as_ref()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    /// Verify Refresh Token
    pub fn verify_refresh_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let claims = Self::verify_token(token)?;
        if claims.token_type != "refresh" {
            return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
        }
        Ok(claims)
    }
}

