mod grpc;

use std::sync::{Arc, RwLock};
use rand::Rng;
use crate::types::{
    game::Game,
    player::Player,
    user::User,
};

#[derive(Debug)]
pub(crate) struct Lobby {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) host_user: Arc<User>,
    pub(super) players: Vec<Arc<Player>>,
    pub(super) game: Option<Arc<Game>>,
}

impl Lobby {
    pub(crate) fn new(name: String, host_user: Arc<User>) -> Arc<RwLock<Self>> {
        let mut rng = rand::thread_rng();
        let random_number: u32 = rng.gen_range(0..=99999999);
        let lobby_id = format!("{:08}", random_number);
        let lobby = Arc::new(
            RwLock::new(
                Self {
                    id: lobby_id,
                    name,
                    host_user,
                    players: vec![],
                    game: None,
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
}


