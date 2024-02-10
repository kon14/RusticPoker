use std::sync::{Weak, RwLock};
use crate::types::lobby::Lobby;

#[derive(Debug)]
pub(crate) struct Game {
    pub(super) lobby: Weak<RwLock<Lobby>>,
}
