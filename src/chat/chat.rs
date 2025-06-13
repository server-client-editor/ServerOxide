use crate::auth::UserId;
use futures_util::stream::{SplitSink, SplitStream};
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use warp::ws::{Message, WebSocket};

#[derive(Debug, Error)]
pub enum ChatError {}

#[async_trait::async_trait]
pub trait ChatService: Send + Sync {
    async fn join_chat(
        &self,
        to_user: SplitSink<WebSocket, Message>,
        from_user: SplitStream<WebSocket>,
        user_id: UserId,
    ) -> Result<(), anyhow::Error>;
}
