use std::sync::Arc;
use crate::captcha::*;
use crate::logger::*;
use crate::settings::Settings;

pub struct Server {
    pub captcha_service: Arc<dyn CaptchaService>,
}

impl Server {
    pub fn try_new(settings: &Settings) -> anyhow::Result<Self> {
        let captcha_service = match settings.captcha.backend.as_str() {
            "fake" => Arc::new(FakeCaptchaService::new()),
            other => return Err(anyhow::anyhow!("Unknown captcha backend: {}", other)),
        };
        debug!(?captcha_service);

        Ok(Self{
            captcha_service,
        })
    }
}