mod grpc;
mod status;

use std::{
    sync::{Arc, RwLock},
    collections::HashSet,
    borrow::Cow,
};
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
    pub(crate) host_user: Arc<User>,
    pub(crate) players: Vec<Arc<Player>>,
    pub(crate) game: Option<Arc<Game>>,
    pub(super) status: LobbyStatus,
    pub(super) matchmaking_acceptance: HashSet<String>, // player name
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
                    matchmaking_acceptance: HashSet::default(),
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
            self.set_status_idle()?;
            Ok(())
        }
    }

    pub(crate) fn rm_player(&mut self, user_id: &String) -> Result<(), Status> {
        let player_index = self.players
            .iter()
            .position(|player| &player.user.upgrade().unwrap().id != user_id);
        if let Some(player_index) = player_index {
            self.players.remove(player_index);
            Ok(())
        } else {
            Err(Status::not_found("User isn't participating in the lobby!"))
        }
    }

    pub(crate) fn set_status_idle(&mut self) -> Result<(), Status> {
        if self.status == LobbyStatus::InGame {
            return Err(
                Status::failed_precondition(
                    "Matchmaking state can't be modified during an active game!"
                )
            )
        }
        self.status = LobbyStatus::Idle;
        self.matchmaking_acceptance.clear();
        Ok(())
    }

    pub(crate) fn get_status(&self) -> &LobbyStatus {
        &self.status
    }

    pub(crate) fn set_status_matchmaking(&mut self) -> Result<(), Status> {
        if self.status == LobbyStatus::InGame {
            return Err(
                Status::failed_precondition(
                    "Matchmaking state can't be modified during an active game!"
                )
            )
        }
        if self.players.len() == 1 {
            return Err(
                Status::failed_precondition(
                    "Matchmaking can't be initiated. Not enough players!"
                )
            )
        }
        self.status = LobbyStatus::Matchmaking;
        self.matchmaking_acceptance.clear();
        self.matchmaking_acceptance.insert(self.host_user.name.clone());
        Ok(())
    }

    pub(crate) fn set_matchmaking_acceptance(&mut self, player_name: Cow<String>, accept: bool) -> Result<(), Status> {
        if self.status != LobbyStatus::Matchmaking {
            return Err(Status::failed_precondition("Lobby not currently matchmaking!"));
        }
        if !self.has_player_name(&player_name) {
            return Err(Status::failed_precondition("Player not participating in lobby!"));
        }
        if accept {
            self.matchmaking_acceptance.insert(player_name.into_owned());
        } else {
            self.matchmaking_acceptance.remove(&*player_name);
        }
        Ok(())
    }
}
