use std::{
    sync::Arc,
    time::SystemTime,
};
use crate::types::user::User;

#[derive(Clone, Debug)]
pub(crate) struct Client {
    pub(super) address: String,
    pub(super) last_heartbeat: SystemTime,
    pub(super) user: Arc<User>,
}

impl Client {
    pub fn new(address: String, user_name: String) -> Self {
        let user = Arc::new(User::new(user_name));
        Self {
            address,
            last_heartbeat: SystemTime::now(),
            user,
        }
    }

    pub fn keep_alive(&mut self) {
        self.last_heartbeat = SystemTime::now();
    }
}
