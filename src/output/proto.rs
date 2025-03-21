use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::service::proto;
use super::structs::*;

impl From<PlayerPublicInfo> for proto::PlayerPublicInfo {
    fn from(info: PlayerPublicInfo) -> Self {
        proto::PlayerPublicInfo {
            player_id: info.player_id.to_string(),
            player_name: info.player_name,
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

impl From<GamePlayerPublicInfoAsPlayer> for proto::game_state::match_state::MatchStatePlayerPublicInfo {
    fn from(info: GamePlayerPublicInfoAsPlayer) -> Self {
        let starting_credits = info.credits.get_starting_credits();
        let pot_credits = info.credits.pot_credits
            .into_iter()
            .map(|(pot_id, credits)| (pot_id.to_string(), credits))
            .collect();
        let hand_cards = info.hand_cards
            .unwrap_or(vec![])
            .into_iter()
            .map(|hand_card| hand_card.into())
            .collect();
        proto::game_state::match_state::MatchStatePlayerPublicInfo {
            player_id: info.player_id.to_string(),
            player_name: info.player_name,
            starting_credits,
            remaining_credits: info.credits.remaining_credits,
            pot_credits,
            hand_cards,
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
        let player_bet_amounts = state.player_bet_amounts
            .map_or_else(HashMap::new, |player_bet_amounts| {
                player_bet_amounts
                    .into_iter()
                    .map(|(player_id, bet)| (player_id.to_string(), bet))
                    .collect()
            });
        let poker_phase = state.poker_phase_specifics.into();
        let table_players_order = state.table_players_order
            .into_iter()
            .map(|player_id| player_id.into())
            .collect();
        let active_player_ids = state.active_player_ids
            .into_iter()
            .map(|player_id| player_id.into())
            .collect();
         proto::game_state::MatchState {
             match_id: state.match_id.to_string(),
             player_info,
             credit_pots,
             player_bet_amounts,
             poker_phase: Some(poker_phase),
             table_players_order,
             active_player_ids,
         }
    }
}

impl From<GameStateAsPlayer> for proto::GameState {
    fn from(state: GameStateAsPlayer) -> Self {
        proto::GameState {
            self_player_id: state.self_player_id.into(),
            lobby_state: Some(state.lobby_state.into()),
            match_state: state.match_state.map(|state| state.into()),
            timestamp: Some(chrono_to_prost_timestamp(state.timestamp)),
        }
    }
}

impl From<LobbyState> for proto::LobbyState {
    fn from(state: LobbyState) -> Self {
        let players = state.players
            .into_iter()
            .map(|player| player.into())
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
            players,
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
            host_player: Some(lobby_info.host_player.into()),
            player_count: lobby_info.player_count,
            status: lobby_info.status as i32,
            settings: Some(lobby_info.settings.into()),
            is_joinable: lobby_info.is_joinable,
        }
    }
}

impl From<MatchStatePhaseSpecificsAsPlayer> for proto::game_state::PokerPhase {
    fn from(phase: MatchStatePhaseSpecificsAsPlayer) -> Self {
        let phase = match phase {
            MatchStatePhaseSpecificsAsPlayer::Ante => proto::game_state::poker_phase::Phase::Ante({}),
            MatchStatePhaseSpecificsAsPlayer::Dealing => proto::game_state::poker_phase::Phase::Dealing({}),
            MatchStatePhaseSpecificsAsPlayer::FirstBetting(phase) => {
                proto::game_state::poker_phase::Phase::FirstBetting(
                    proto::game_state::poker_phase::PokerPhaseBetting {
                        highest_bet_amount: Some(phase.highest_bet_amount),
                        self_bet_amount: Some(phase.self_bet_amount),
                    }
                )
            }
            MatchStatePhaseSpecificsAsPlayer::Drawing(phase) => {
                proto::game_state::poker_phase::Phase::Drawing({
                    proto::game_state::poker_phase::PokerPhaseDrawing {
                        stage: Some(phase.into()),
                    }
                })
            }
            MatchStatePhaseSpecificsAsPlayer::SecondBetting(phase) => {
                proto::game_state::poker_phase::Phase::SecondBetting(
                    proto::game_state::poker_phase::PokerPhaseBetting {
                        highest_bet_amount: Some(phase.highest_bet_amount),
                        self_bet_amount: Some(phase.self_bet_amount),
                    }
                )
            }
            MatchStatePhaseSpecificsAsPlayer::Showdown(phase) => {
                let winning_rank: proto::game_state::poker_phase::poker_phase_showdown::showdown_results::PokerHandRank = phase.winning_rank.into();
                let pot_distribution = phase.pot_distribution
                    .into_values()
                    .map(|distribution| distribution.into())
                    .collect();
                proto::game_state::poker_phase::Phase::Showdown({
                    let winner_ids = phase.winner_ids
                        .into_iter()
                        .map(|winner_id| winner_id.into())
                        .collect();
                    let results = proto::game_state::poker_phase::poker_phase_showdown::ShowdownResults {
                        winning_rank: winning_rank as i32,
                        winner_ids,
                        pot_distribution,
                    };
                    proto::game_state::poker_phase::PokerPhaseShowdown {
                        results: Some(results),
                    }
                })
            }
        };
        proto::game_state::PokerPhase {
            phase: Some(phase),
        }
    }
}

impl From<ShowdownPotDistribution> for proto::game_state::poker_phase::poker_phase_showdown::showdown_results::ShowdownPotDistribution {
    fn from(distribution: ShowdownPotDistribution) -> Self {
        Self {
            pot_id: distribution.pot_id.into(),
            player_ids: distribution.player_ids.into_iter().map(|player_id| player_id.into()).collect(),
            total_credits: distribution.total_credits,
            credits_per_winner: distribution.credits_per_winner,
        }
    }
}

impl From<MatchStatePhaseSpecificsDrawingAsPlayer> for proto::game_state::poker_phase::poker_phase_drawing::DrawingStage {
    fn from(drawing_stage: MatchStatePhaseSpecificsDrawingAsPlayer) -> Self {
        let proto_stage = match drawing_stage {
            MatchStatePhaseSpecificsDrawingAsPlayer::Discarding(discard_stage) => {
                let player_discard_count = discard_stage.player_discard_count
                    .into_iter()
                    .map(|(player_id, discard_count)| (player_id.into(), discard_count as u32))
                    .collect();
                let proto_discard_stage = proto::game_state::poker_phase::poker_phase_drawing::drawing_stage::DrawingStageDiscarding {
                    player_discard_count,
                };
                proto::game_state::poker_phase::poker_phase_drawing::drawing_stage::Stage::Discarding(proto_discard_stage)
            },
            MatchStatePhaseSpecificsDrawingAsPlayer::Dealing => proto::game_state::poker_phase::poker_phase_drawing::drawing_stage::Stage::Dealing(()),
        };
        Self {
            stage: Some(proto_stage)
        }
    }
}

impl From<HandCard> for proto::game_state::match_state::match_state_player_public_info::HandCard {
    fn from(card: HandCard) -> Self {
        let inner_card = match card {
            HandCard::VisibleCard(card) => proto::game_state::match_state::match_state_player_public_info::hand_card::Card::VisibleCard(card.into()),
            HandCard::HiddenCard => proto::game_state::match_state::match_state_player_public_info::hand_card::Card::HiddenCard(()),
            HandCard::DiscardedCard => proto::game_state::match_state::match_state_player_public_info::hand_card::Card::DiscardedCard(()),
        };
        Self {
            card: Some(inner_card),
        }
    }
}

fn chrono_to_prost_timestamp(dt: DateTime<Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}
