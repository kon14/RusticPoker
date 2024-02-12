mod grpc;

use std::sync::{Arc, RwLock};
use tonic::Status;
use crate::types::{
    game::Game,
    player::Player,
    user::User,
};
use crate::service::proto::LobbyStatus;

#[derive(Debug)]
pub(crate) struct Lobby {
    pub(crate) id: String,
    pub(super) name: String,
    pub(super) host_user: Arc<User>,
    pub(super) players: Vec<Arc<Player>>,
    pub(super) game: Option<Arc<Game>>,
    pub(super) status: LobbyStatus,
}

impl Lobby {
    pub(crate) fn new(id: String, name: String, host_user: Arc<User>) -> Arc<RwLock<Self>> {
        let lobby = Arc::new(
            RwLock::new(
                Self {
                    id,
                    name,
                    host_user,
                    players: vec![],
                    game: None,
                    status: LobbyStatus::Idle,
                }
            )
        );
        let host_player = Arc::new(
            Player {
                user: Arc::downgrade(&lobby.read().unwrap().host_user),
                lobby: Arc::downgrade(&lobby),
            }
        );
        lobby.write().unwrap().players.push(host_player);
        lobby
    }

    pub(crate) fn has_player_name(&self, player_name: &String) -> bool {
        self.players
            .iter()
            .any(|player| &player.user.upgrade().unwrap().name == player_name)
    }

    pub(crate) fn add_player(&mut self, player: Arc<Player>) -> Result<(), Status> {
        let player_name = &player.user.upgrade().unwrap().name;
        if self.has_player_name(player_name) {
            Err(Status::already_exists("User is already a member of the lobby!"))
        } else {
            self.players.push(player);
            Ok(())
        }
    }
}
