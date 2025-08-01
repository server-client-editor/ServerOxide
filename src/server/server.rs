use std::sync::Arc;
use crate::auth::*;
use crate::captcha::*;
use crate::chat::*;
use crate::logger::*;
use crate::user::*;
use crate::settings::Settings;

pub struct Server {
    pub auth_service: Arc<dyn AuthService>,
    pub captcha_service: Arc<dyn CaptchaService>,
    pub chat_service: Arc<dyn ChatService>,
    pub user_service: Arc<dyn UserService>,
}

impl Server {
    pub fn try_new(settings: &Settings) -> anyhow::Result<Self> {
        let captcha_service = match settings.captcha.backend.as_str() {
            "fake" => Arc::new(FakeCaptchaService::new()),
            other => return Err(anyhow::anyhow!("Unknown captcha backend: {}", other)),
        };
        debug!(?captcha_service);

        let auth_service = match settings.auth.backend.as_str() {
            "fake" => Arc::new(FakeAuthService::new()),
            other => return Err(anyhow::anyhow!("Unknown auth backend: {}", other)),
        };
        debug!(?auth_service);

        let user_service = match settings.user.backend.as_str() {
            "fake" => Arc::new(FakeUserService::new()),
            other => return Err(anyhow::anyhow!("Unknown user backend: {}", other)),
        };
        debug!(?user_service);

        let chat_service = match settings.chat.backend.as_str() {
            "fake" => Arc::new(FakeChatService::new(user_service.clone())),
            other => return Err(anyhow::anyhow!("Unknown chat backend: {}", other)),
        };
        debug!(?chat_service);

        Ok(Self{
            auth_service,
            captcha_service,
            chat_service,
            user_service,
        })
    }
}