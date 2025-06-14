use serde::{Serialize};
use thiserror::Error;
use crate::domain::UserId;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid username or password")]
    InvalidCredentials,
    #[error("Username already taken")]
    UsernameTaken,
    #[error("Token is not valid")]
    InvalidToken,
    #[error("token expired")]
    TokenExpired,
    #[error("refresh token is not valid")]
    InvalidRefreshToken,
    #[error("refresh token expired")]
    RefreshTokenExpired,
    #[error("internal error: {0}")]
    InternalError(#[from] anyhow::Error),
}

#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub access_expires_in: u64,  // seconds
    pub refresh_token: String,
    pub refresh_expires_in: u64,  // seconds
}

#[derive(Debug)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct LoginResult {
    pub user_id: UserId,
    pub auth_tokens: AuthTokens,
}

#[derive(Debug)]
pub struct SignupInput {
    pub username: String,
    pub password: String,
}

#[async_trait::async_trait]
pub trait AuthService: Send + Sync {
    async fn login(&self, request: LoginInput) -> Result<LoginResult, AuthError>;
    async fn signup(&self, request: SignupInput) -> Result<UserId, AuthError>;
    async fn verify_token(&self, token: &str) -> Result<UserId, AuthError>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthTokens, AuthError>;
}