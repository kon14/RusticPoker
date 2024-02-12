use crate::{
    service::proto::{
        LobbyInfoPublic,
        LobbyInfoPrivate,
    },
    types::lobby::Lobby,
};

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

impl Into<LobbyInfoPrivate> for &Lobby {
    fn into(self) -> LobbyInfoPrivate {
        LobbyInfoPrivate {
            id: self.id.clone(),
            name: self.name.clone(),
            host_user: self.host_user.name.clone(),
            players: self.players.iter().map(|player| player.as_ref().into()).collect(),
            status: self.status.into(),
        }
    }
}
