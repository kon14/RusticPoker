mod grpc;

use std::sync::{Weak, RwLock};
use crate::types::{
    lobby::Lobby,
    user::User,
};

#[derive(Debug)]
pub(crate) struct Player {
    pub(crate) user: Weak<User>,
    pub(crate) lobby: Weak<RwLock<Lobby>>,
}
