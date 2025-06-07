use crate::captcha::*;
use crate::logger::*;
use chrono::{DateTime, Utc};
use futures_util::sink::SinkExt;
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;
use warp::{self, reject};

/// TODO: This is currently a God File to help us move fast.
/// Refactor and tidy up when the feature set is more stable.

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid captcha ID or answer")]
    InvalidCaptcha,
    #[error("Internal error")]
    InternalError,
}

impl reject::Reject for ApiError {}

fn map_captcha_error_to_api_error(e: CaptchaError) -> ApiError {
    match e {
        CaptchaError::Mismatch => ApiError::InvalidCaptcha,
        CaptchaError::NotFound => ApiError::InvalidCaptcha,
        CaptchaError::InternalError(e) => {
            warn!("Internal captcha error: {}", e);
            ApiError::InternalError
        },
    }
}

#[derive(Debug, Serialize)]
struct CaptchaResponse {
    id: uuid::Uuid,
    image_base64: String,
    expire_at: DateTime<Utc>,
}

pub async fn generate_captcha(
    captcha_service: Arc<dyn CaptchaService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let captcha = captcha_service
        .generate()
        .await
        .map_err(map_captcha_error_to_api_error)
        .map_err(reject::custom)?;

    let response = CaptchaResponse {
        id: captcha.id,
        image_base64: captcha.image_base64,
        expire_at: captcha.expire_at,
    };
    Ok(warp::reply::json(&response))
}
pub async fn login(body: serde_json::Value) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(format!("Login ({})\n", body))
}
pub async fn signup(body: serde_json::Value) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(format!("Signup ({})\n", body))
}
pub async fn join_chat(mut socket: warp::ws::WebSocket) {
    let greeting = "Hello from WebSocket!\n".to_string();
    let _ = socket.send(warp::ws::Message::text(greeting)).await;
    let _ = socket.close();
}
