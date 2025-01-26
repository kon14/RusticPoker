use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::error::AppError;
use crate::lobby::{Lobby, LobbySettings};
use crate::player::Player;
use crate::r#match::Match;
use crate::service::proto;

#[derive(Clone, Debug)]
pub struct GameState {
    player_states: HashMap<Uuid, PlayerState>,
    lobby_state: LobbyState,
    match_state: Option<MatchState>,
    timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct GameStateAsPlayer {
    player_state: PlayerState,
    lobby_state: LobbyState,
    match_state: Option<MatchState>, // hide sensitive info (table) TODO
    timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    player_id: Uuid,
    // name: String,
    total_credits: u64,
}

impl From<Player> for PlayerState {
    fn from(player: Player) -> Self {
        PlayerState {
            player_id: player.player_id,
            total_credits: player.total_credits,
        }
    }
}

impl From<PlayerState> for proto::PlayerState {
    fn from(state: PlayerState) -> Self {
        proto::PlayerState {
            player_id: state.player_id.to_string(),
            total_credits: state.total_credits,
        }
    }
}

#[derive(Clone, Debug)]
pub enum LobbyStatus {
    Idle,
    Matchmaking,
    InGame,
}

impl From<&Lobby> for LobbyStatus {
    fn from(lobby: &Lobby) -> Self {
        match (lobby.is_matchmaking(), lobby.is_in_game()) {
            (false, false) => LobbyStatus::Idle,
            (true, false) => LobbyStatus::Matchmaking,
            (false, true) => LobbyStatus::InGame,
            _ => unreachable!(),
        }
    }
}

impl From<LobbyStatus> for proto::LobbyStatus {
    fn from(status: LobbyStatus) -> Self {
        match status {
            LobbyStatus::Idle => proto::LobbyStatus::Idle,
            LobbyStatus::Matchmaking => proto::LobbyStatus::Matchmaking,
            LobbyStatus::InGame => proto::LobbyStatus::InGame,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LobbyState {
    lobby_id: Uuid,
    name: String,
    host_player_id: Uuid,
    player_ids: HashSet<Uuid>,
    status: LobbyStatus,
    game_acceptance: HashMap<Uuid, bool>,
    settings: LobbySettings,
}

impl From<Lobby> for LobbyState {
    fn from(lobby: Lobby) -> Self {
        let status: LobbyStatus = (&lobby).into();
        let game_acceptance =
            lobby
                .player_ids
                .iter()
                .map(|player_id| {
                    let is_accepted = lobby.game_acceptance
                        .as_ref()
                        .map_or(false, |set| set.contains(player_id));
                    (*player_id, is_accepted)
                })
                .collect::<HashMap<_, _>>();
        LobbyState {
            lobby_id: lobby.lobby_id,
            name: lobby.name,
            host_player_id: lobby.host_player_id,
            player_ids: lobby.player_ids,
            status,
            game_acceptance,
            settings: lobby.settings,
        }
    }
}

impl From<LobbyState> for proto::LobbyState {
    fn from(state: LobbyState) -> Self {
        let player_ids = state.player_ids
            .into_iter()
            .map(|player_id| player_id.to_string())
            .collect();
        let game_acceptance = state.game_acceptance
            .into_iter()
            .map(|(player_id, acceptance)| (player_id.to_string(), acceptance))
            .collect();
        let status: proto::LobbyStatus = state.status.into();
        proto::LobbyState {
            lobby_id: state.lobby_id.to_string(),
            name: state.name,
            host_player_id: state.host_player_id.to_string(),
            player_ids,
            status: status as i32,
            game_acceptance,
            settings: Some(state.settings.into()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MatchState {
    match_id: Uuid,
}

impl From<Match> for MatchState {
    fn from(r#match: Match) -> Self {
        MatchState {
            match_id: r#match.match_id,
        }
    }
}

impl From<MatchState> for proto::MatchState {
    fn from(state: MatchState) -> Self {
        proto::MatchState {
            match_id: state.match_id.to_string(),
        }
    }
}

impl GameState {
    pub fn build(
        player_states: HashMap<Uuid, PlayerState>,
        lobby_state: LobbyState,
        match_state: Option<MatchState>,
    ) -> Self {
        GameState {
            player_states,
            lobby_state,
            match_state,
            timestamp: Utc::now(),
        }
    }

    pub fn as_player(&self, player_id: &Uuid) -> Result<GameStateAsPlayer, AppError> {
        let Some(player_state) = self.player_states.get(player_id) else {
            return Err(AppError::unauthorized(
                format!("Player ({}) not participating in lobby ({})!", player_id, self.lobby_state.lobby_id)
            ));
        };


        let player_state = player_state.clone();
        let lobby_state = self.lobby_state.clone();
        let match_state = self.match_state.clone(); // TODO: hide sensitive data

        let state = GameStateAsPlayer {
            player_state,
            lobby_state,
            match_state,
            timestamp: self.timestamp,
        };
        Ok(state)
    }
}

impl From<GameStateAsPlayer> for proto::GameState {
    fn from(state: GameStateAsPlayer) -> Self {
        proto::GameState {
            player_state: Some(state.player_state.into()),
            lobby_state: Some(state.lobby_state.into()),
            match_state: state.match_state.map(|state| state.into()),
            timestamp: Some(chrono_to_prost_timestamp(state.timestamp)),
        }
    }
}

fn chrono_to_prost_timestamp(dt: DateTime<Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}
