use crate::service::proto::{
    LobbyStatus,
    set_lobby_matchmaking_status_request::MatchmakingStatus,
};

impl Into<LobbyStatus> for MatchmakingStatus {
    fn into(self) -> LobbyStatus {
        match self {
            MatchmakingStatus::Matchmaking => LobbyStatus::Matchmaking,
            MatchmakingStatus::NotMatchmaking => LobbyStatus::Idle,
        }
    }
}
