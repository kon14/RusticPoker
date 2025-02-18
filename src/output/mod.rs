mod broadcaster;
mod proto;
mod structs;

pub(crate) use broadcaster::GameStateBroadcaster;
pub(crate) use structs::{GameState, GameStateAsPlayer, LobbyInfoPublic};

use std::collections::HashMap;
use chrono::Utc;
use itertools::Itertools;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::lobby::Lobby;
use crate::player::Player;
use crate::r#match::Match;
use structs::*;

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
        let mut own_match_state = None;
        if let Some(match_state) = self.match_state.as_ref() {
            own_match_state = Some(match_state.as_player(player_id)?.clone())
        };

        let state = GameStateAsPlayer {
            player_state,
            lobby_state,
            match_state: own_match_state,
            timestamp: self.timestamp,
        };
        Ok(state)
    }
}


impl MatchState {
    pub fn as_player(&self, player_id: &Uuid) -> Result<MatchStateAsPlayer, AppError> {
        let player_cards = self.player_cards
            .as_ref()
            .map(|player_cards| {
                player_cards
                    .get(player_id)
                    .map(|hand| hand.clone())
                    .flatten()
                    .ok_or_else(|| AppError::unauthorized(
                        format!("Player ({}) not participating in match ({})!", player_id, self.match_id)
                    ))
            })
            .transpose()?;

        let state = MatchStateAsPlayer {
            match_id: self.match_id,
            player_info: self.player_info.clone(),
            player_cards,
            credit_pots: self.credit_pots.clone(),
        };
        Ok(state)
    }
}

impl GamePlayerPublicInfo {
    pub async fn from_match(r#match: &Match) -> HashMap<Uuid, Self> {
        let mut game_phase_w = r#match.phase.write().await;
        let player_credits = game_phase_w
            .get_table()
            .player_credits
            .clone();

        let cards_in_hand = &game_phase_w
            .get_player_cards();

        player_credits
            .into_iter()
            .map(|(player_id, self_credits)| {
                let self_hand_card_count = cards_in_hand
                    .as_ref()
                    .and_then(|map| map.get(&player_id))
                    .and_then(|cards_option| cards_option.as_ref())
                    .map_or(0, |cards| cards.len() as u8);
                let info = GamePlayerPublicInfo {
                    player_id: player_id.clone(),
                    credits: self_credits,
                    hand_card_count: self_hand_card_count,
                };
                (player_id, info)
            })
            .collect()
    }
}

impl From<Player> for PlayerState {
    fn from(player: Player) -> Self {
        PlayerState {
            player_id: player.player_id,
            name: "Anonymous".to_string(), // player.name, // TODO
        }
    }
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

impl MatchState {
    pub(crate) async fn from_match(r#match: Match) -> Self {
        let player_info = GamePlayerPublicInfo::from_match(&r#match).await;
        let mut game_phase_w = r#match.phase.write().await;
        let player_cards = game_phase_w.get_player_cards();
        let credit_pots = &game_phase_w.get_table().credit_pots;
        MatchState {
            match_id: r#match.match_id,
            player_info,
            player_cards,
            credit_pots: credit_pots.clone()
        }
    }
}

impl From<Lobby> for LobbyInfoPublic {
    fn from(lobby: Lobby) -> Self {
        let status = LobbyStatus::from(&lobby);
        LobbyInfoPublic {
            lobby_id: lobby.lobby_id,
            name: lobby.name,
            host_player_id: lobby.host_player_id,
            player_count: lobby.player_ids.len().try_into().unwrap(),
            status,
        }
    }
}
