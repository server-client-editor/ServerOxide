use anyhow::Result;
use thiserror::Error;
use crate::domain::{ConversationId, UserId};

#[derive(Debug, Error)]
pub enum UserServiceError {

}

#[async_trait::async_trait]
pub trait UserService: Send + Sync {
    async fn get_receiver(&self, user_id: &UserId, conversation_id: &ConversationId) -> Result<Vec<UserId>>;
}
