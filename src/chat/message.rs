use serde::{Serialize, Deserialize};
use crate::domain::{ConversationId, UserId};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "lowercase")]
pub enum ClientToServer {
    HistoryFetched,
    Send(SendMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessage {
    #[serde(flatten)]
    pub content: ChatContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "lowercase")]
pub enum ServerToClient {
    Distribute(DistributeMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DistributeMessage {
    pub sender: UserId,
    #[serde(flatten)]
    pub content: ChatContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatContent {
    pub conversation_id: ConversationId,
    pub content: String,
}
