use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::service::proto;
use super::structs::*;

impl From<PlayerState> for proto::PlayerState {
    fn from(state: PlayerState) -> Self {
        proto::PlayerState {
            player_id: state.player_id.to_string(),
            name: state.name,
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

impl From<GamePlayerPublicInfo> for proto::game_state::match_state::MatchStatePlayerPublicInfo {
    fn from(info: GamePlayerPublicInfo) -> Self {
        let starting_credits = info.credits.get_starting_credits();
        let pot_credits = info.credits.pot_credits
            .into_iter()
            .map(|(pot_id, credits)| (pot_id.to_string(), credits))
            .collect();
        proto::game_state::match_state::MatchStatePlayerPublicInfo {
            player_id: info.player_id.to_string(),
            starting_credits,
            remaining_credits: info.credits.remaining_credits,
            pot_credits,
            hand_card_count: info.hand_card_count as u32,
        }
    }
}

impl From<MatchStateAsPlayer> for proto::game_state::MatchState {
    fn from(state: MatchStateAsPlayer) -> Self {
        let player_info = state.player_info
            .into_iter()
            .map(|(player_id, player_info)| (player_id.into(), player_info.into()))
            .collect();
        let credit_pots = state.credit_pots
            .into_iter()
            .map(|(pot_id, pot)| (pot_id.to_string(), pot.into()))
            .collect();
        let own_cards = state.player_cards
            .map(|cards| cards.into_iter().map(|card| card.into()).collect::<Vec<_>>())
            .unwrap_or_else(|| Vec::with_capacity(0));
        let player_bet_amounts = state.player_bet_amounts
            .map_or_else(HashMap::new, |player_bet_amounts| {
                player_bet_amounts
                    .into_iter()
                    .map(|(player_id, bet)| (player_id.to_string(), bet))
                    .collect()
            });
        // let poker_phase = todo!();
        proto::game_state::MatchState {
            match_id: state.match_id.to_string(),
            player_info,
            credit_pots,
            own_cards,
            player_bet_amounts,
            // poker_phase,
        }
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

impl From<LobbyInfoPublic> for proto::LobbyInfoPublic {
    fn from(lobby_info: LobbyInfoPublic) -> Self {
        proto::LobbyInfoPublic {
            lobby_id: lobby_info.lobby_id.to_string(),
            name: lobby_info.name,
            host_player_id: lobby_info.host_player_id.to_string(),
            player_count: lobby_info.player_count,
            status: lobby_info.status as i32,
        }
    }
}

fn chrono_to_prost_timestamp(dt: DateTime<Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}
