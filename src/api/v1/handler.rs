use super::error::*;
use crate::auth::*;
use crate::captcha::*;
use crate::logger::*;
use crate::chat::ChatService;
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::{self, reject};

/// TODO: This is currently a God File to help us move fast.
/// Refactor and tidy up when the feature set is more stable.

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

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub captcha_id: uuid::Uuid,
    pub captcha_answer: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user_id: UserId,
    pub auth_tokens: AuthTokens,
}

pub async fn login(
    body: LoginRequest,
    auth_service: Arc<dyn AuthService>,
    captcha_service: Arc<dyn CaptchaService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let validation_input = ValidationInput {
        id: body.captcha_id,
        answer: body.captcha_answer,
    };
    captcha_service
        .validate(validation_input)
        .await
        .map_err(map_captcha_error_to_api_error)
        .map_err(reject::custom)?;

    let login_input = LoginInput {
        username: body.username.clone(),
        password: body.password.clone(),
    };
    let login_result = auth_service
        .login(login_input)
        .await
        .map_err(map_auth_error_to_api_error)
        .map_err(reject::custom)?;

    Ok(warp::reply::json(&LoginResponse {
        user_id: login_result.user_id,
        auth_tokens: login_result.auth_tokens,
    }))
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
    pub captcha_id: uuid::Uuid,
    pub captcha_answer: String,
}

#[derive(Debug, Serialize)]
pub struct SignupResponse;

pub async fn signup(
    body: SignupRequest,
    auth_service: Arc<dyn AuthService>,
    captcha_service: Arc<dyn CaptchaService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let validation_input = ValidationInput {
        id: body.captcha_id,
        answer: body.captcha_answer,
    };
    captcha_service
        .validate(validation_input)
        .await
        .map_err(map_captcha_error_to_api_error)
        .map_err(reject::custom)?;

    let signup_input = SignupInput {
        username: body.username,
        password: body.password,
    };
    let _user_id = auth_service
        .signup(signup_input)
        .await
        .map_err(map_auth_error_to_api_error)
        .map_err(reject::custom)?;

    Ok(warp::reply::json(&SignupResponse))
}
pub async fn join_chat(
    socket: warp::ws::WebSocket,
    user_id: UserId,
    chat_service: Arc<dyn ChatService>,
) {
    let (to_user, from_user) = socket.split();
    if let Err(e) = chat_service.join_chat(to_user, from_user, user_id).await {
        error!("Error joining chat: {}", e);
    }
}
