use std::{
    sync::{Arc, Weak, RwLock},
    time::SystemTime,
};
use crate::types::{
    user::User,
    lobby::Lobby,
};

#[derive(Clone, Debug)]
pub(crate) struct Client {
    pub(super) address: String,
    pub(super) last_heartbeat: SystemTime,
    pub(super) user: Arc<User>,
    pub(super) lobby: Option<Weak<RwLock<Lobby>>>,
}

impl Client {
    pub fn new(address: String, user_id: String, user_name: String) -> Self {
        let user = Arc::new(User::new(user_id, user_name));
        Self {
            address,
            last_heartbeat: SystemTime::now(),
            user,
            lobby: None,
        }
    }

    pub fn keep_alive(&mut self) {
        self.last_heartbeat = SystemTime::now();
    }
}
