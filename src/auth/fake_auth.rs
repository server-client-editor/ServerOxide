use crate::auth::*;

#[derive(Debug)]
pub struct FakeAuthService;

impl FakeAuthService {
    pub fn new() -> Self {
        Self
    }
}

// Minimal fake implementation for basic use only.
// Extend to simulate more error cases and configurable responses when needed.
#[async_trait::async_trait]
impl AuthService for FakeAuthService {
    async fn login(&self, request: LoginInput) -> Result<LoginResult, AuthError> {
        Ok(LoginResult {
            user_id: get_fake_id(&request.username),
            auth_tokens: get_fake_token(&request.username),
        })
    }

    async fn signup(&self, request: SignupInput) -> Result<UserId, AuthError> {
        Ok(get_fake_id(&request.username))
    }

    async fn verify_token(&self, token: &str) -> Result<UserId, AuthError> {
        if let Some(username) = token.strip_prefix("fake-access-token:") {
            Ok(get_fake_id(&username))
        } else {
            Err(AuthError::InvalidToken)
        }
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthTokens, AuthError> {
        if let Some(username) = refresh_token.strip_prefix("fake-refresh-token:") {
            Ok(get_fake_token(&username))
        } else {
            Err(AuthError::InvalidRefreshToken)
        }
    }
}

fn get_fake_id(username: &str) -> UserId {
    UserId(uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, username.as_bytes()))
}

fn get_fake_token(username: &str) -> AuthTokens {
    AuthTokens {
        access_token: format!("fake-access-token:{}", username),
        access_expires_in: 1 * 60 * 60,  // 1 hour
        refresh_token: format!("fake-refresh-token:{}", username),
        refresh_expires_in: 7 * 24 * 60 * 60,  // 7 days
    }
}