use thiserror::Error;
use tracing::warn;
use warp::reject;
use crate::auth::AuthError;
use crate::captcha::CaptchaError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid captcha ID or answer")]
    InvalidCaptcha,
    #[error("Invalid username or password")]
    InvalidCredentials,
    #[error("Username already taken")]
    UsernameTaken,
    #[error("Token is not valid")]
    InvalidToken,
    #[error("Internal error")]
    InternalError,
}

impl reject::Reject for ApiError {}

pub fn map_captcha_error_to_api_error(e: CaptchaError) -> ApiError {
    match e {
        CaptchaError::Mismatch => ApiError::InvalidCaptcha,
        CaptchaError::NotFound => ApiError::InvalidCaptcha,
        CaptchaError::InternalError(e) => {
            warn!("Internal captcha error: {}", e);
            ApiError::InternalError
        }
    }
}

pub fn map_auth_error_to_api_error(e: AuthError) -> ApiError {
    match e {
        AuthError::InvalidCredentials => ApiError::InvalidCredentials,
        AuthError::UsernameTaken => ApiError::UsernameTaken,
        // These error variants are grouped under InvalidToken for now.
        // Will need finer-grained mapping when JWT support is implemented.
        AuthError::InvalidToken
        | AuthError::TokenExpired
        | AuthError::InvalidRefreshToken
        | AuthError::RefreshTokenExpired => ApiError::InvalidToken,
        AuthError::InternalError(e) => {
            warn!("Internal auth error: {}", e);
            ApiError::InternalError
        }
    }
}