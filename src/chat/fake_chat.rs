use std::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use crate::auth::UserId;
use super::chat::*;
use crate::logger::*;
use crate::user::*;
use futures_util::stream::{SplitSink, SplitStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use warp::ws::{Message, WebSocket};

pub struct ClientRecord {
    pub user_id: UserId,
    pub to_sender: UnboundedSender<Message>,
    pub watcher_handle: JoinHandle<()>,
}

impl Debug for FakeChatService {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeChatService")
            .field("online_users", &self.online_users.len())
            .finish()
    }
}

pub struct FakeChatService {
    user_service: Arc<dyn UserService>,
    online_users: Arc<DashMap<UserId, ClientRecord>>,
    to_dispatcher: UnboundedSender<Message>,
    dispatcher_handle: JoinHandle<()>,
}

fn new_dispatcher(mut from_receiver: UnboundedReceiver<Message>, online_users: Arc<DashMap<UserId, ClientRecord>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(message) = from_receiver.recv().await {
            for elem in online_users.iter() {
                let _ = elem.to_sender.send(message.clone());
            }
        }
    })
}

impl FakeChatService {
    pub fn new(user_service: Arc<dyn UserService>) -> Self {
        let (to_dispatcher, from_receiver) = unbounded_channel();
        let online_users = Arc::new(DashMap::new());
        let dispatcher_handle = new_dispatcher(from_receiver, online_users.clone());

        Self {
            user_service,
            online_users,
            to_dispatcher,
            dispatcher_handle
        }
    }
}

#[async_trait::async_trait]
impl ChatService for FakeChatService {
    async fn join_chat(
        &self,
        mut to_user: SplitSink<WebSocket, Message>,
        mut from_user: SplitStream<WebSocket>,
        user_id: UserId,
    ) -> Result<(), anyhow::Error> {
        let to_dispatcher = self.to_dispatcher.clone();
        let receiver_handle = tokio::task::spawn(async move {
            while let Some(result) = from_user.next().await {
                if let Ok(message) = result {
                    let message = format!("{}: {}", user_id.0, message.to_str().unwrap_or_default());
                    let _ = to_dispatcher.send(Message::text(message));
                } else {
                    break;
                }
            }
        });

        let (to_sender, mut from_dispatcher) = unbounded_channel();
        let sender_handle = tokio::task::spawn(async move {
            while let Some(message) = from_dispatcher.recv().await {
                if let Err(_) = to_user.send(message).await {
                    break;
                }
            }
        });

        let user_id_clone = user_id.clone();
        let online_users_clone = self.online_users.clone();
        let watcher_handle = tokio::task::spawn(async move {
            let _ = tokio::join!(sender_handle, receiver_handle);
            online_users_clone.remove(&user_id_clone);
            debug!("online_users: {}", online_users_clone.len());
        });

        let user_id_clone = user_id.clone();
        let new_user = ClientRecord {
            user_id,
            to_sender,
            watcher_handle,
        };
        self.online_users.insert(user_id_clone, new_user);
        debug!("online_users: {}", self.online_users.len());

        Ok(())
    }
}
