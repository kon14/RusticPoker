mod grpc;

use std::sync::{Weak, RwLock};
use crate::types::{
    lobby::Lobby,
    user::User,
};

#[derive(Debug)]
pub(crate) struct Player {
    pub(super) user: Weak<User>,
    pub(super) lobby: Weak<RwLock<Lobby>>,
}
