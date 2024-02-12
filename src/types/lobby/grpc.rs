use std::collections::HashMap;
use crate::service::proto::{
    LobbyInfoPublic,
    LobbyInfoPrivate,
    LobbyStatus,
};
use crate::types::lobby::Lobby;

impl Into<LobbyInfoPublic> for &Lobby {
    fn into(self) -> LobbyInfoPublic {
        LobbyInfoPublic {
            id: self.id.clone(),
            name: self.name.clone(),
            host_user: self.host_user.name.clone(),
            player_count: self.players.len() as u32,
        }
    }
}

impl From<&Lobby> for LobbyInfoPrivate {
    fn from(value: &Lobby) -> Self {
        let mut matchmaking_acceptance: HashMap<String, bool> = HashMap::default();
        if value.status == LobbyStatus::Matchmaking {
            matchmaking_acceptance = value.players
                .iter()
                .map(|player| player.user.upgrade().unwrap().name.clone())
                .map(|player_name| {
                    let accept = value.matchmaking_acceptance.contains(&player_name);
                    (player_name, accept)
                })
                .collect();
        }
        LobbyInfoPrivate {
            id: value.id.clone(),
            name: value.name.clone(),
            host_user: value.host_user.name.clone(),
            players: value.players.iter().map(|player| player.as_ref().into()).collect(),
            status: value.status.into(),
            matchmaking_acceptance,
        }
    }
}
