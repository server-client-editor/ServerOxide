use std::time::Duration;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptchaError {
    #[error("Captcha mismatch")]
    Mismatch,
    #[error("Captcha not found or expired")]
    NotFound,
    #[error("Internal error: {0}")]
    InternalError(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct CaptchaResult {
    pub id: uuid::Uuid,
    pub image_base64: String,
    pub expire_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ValidationInput {
    pub id: uuid::Uuid,
    pub answer: String,
}

#[async_trait::async_trait]
pub trait CaptchaService: Send + Sync {
    /// Generate a captcha image that will expire after the given duration.
    async fn generate(&self) -> Result<CaptchaResult, CaptchaError>;

    /// Validate the user's answer to a captcha.
    /// Returns Ok(true) if valid, Ok(false) if invalid, Err if internal error.
    async fn validate(&self, input: ValidationInput) -> Result<(), CaptchaError>;
}