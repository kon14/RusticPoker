use std::time::SystemTime;

#[derive(Clone, Debug)]
pub(crate) struct Client {
    pub(super) address: String,
    pub(super) last_heartbeat: SystemTime,
    pub(super) player_name: String,
}

impl Client {
    pub fn new(address: String, player_name: String) -> Self {
        Self {
            address,
            last_heartbeat: SystemTime::now(),
            player_name,
        }
    }

    pub fn keep_alive(&mut self) {
        self.last_heartbeat = SystemTime::now();
    }
}
