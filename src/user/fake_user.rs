use std::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use dashmap::DashMap;
use uuid::Uuid;
use crate::domain::{ConversationId, UserId};
use crate::user::UserService;

impl Debug for FakeUserService {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeUserService").finish()
    }
}

pub struct FakeUserService {
    pub users: Arc<DashMap<i32, UserId>>,
    pub indices: Arc<DashMap<UserId, i32>>,
}

impl FakeUserService {
    pub fn new() -> Self {
        let users = Arc::new(DashMap::new());
        let indices: Arc<DashMap<UserId, i32>> = Arc::new(DashMap::new());
        for i in 0..10 {
            let username = format!("testuser{}", i);
            let user_id = Uuid::new_v5(&Uuid::NAMESPACE_OID, username.as_bytes());
            users.insert(i, UserId(user_id));
            indices.insert(UserId(user_id), i);
        }

        Self {
            users,
            indices,
        }
    }

    fn get_user_id(&self, index: i32) -> Result<UserId> {
        Ok(self.users.get(&index).ok_or(anyhow!("User index not found: {}", index))?.clone())
    }
    fn get_index(&self, user_id: &UserId) -> Result<i32> {
        Ok(self.indices.get(user_id).ok_or(anyhow!("User ID not found: {:?}", user_id))?.clone())
    }
}

#[async_trait::async_trait]
impl UserService for FakeUserService {
    async fn get_receiver(&self, user_id: &UserId, _conversation_id: &ConversationId) -> Result<Vec<UserId>> {
        let index = self.get_index(user_id)?;
        match index {
            0 => Ok(vec![self.get_user_id(1)?]),
            1 => Ok(vec![self.get_user_id(0)?]),
            2 => Ok(vec![self.get_user_id(0)?, self.get_user_id(1)?]),
            _ => Err(anyhow!("User connections not found for index {}", index))
        }
    }
}
