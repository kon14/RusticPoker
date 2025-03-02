mod broadcaster;
mod proto;
mod structs;

pub(crate) use broadcaster::GameStateBroadcaster;
pub(crate) use structs::{GameState, GameStateAsPlayer, LobbyInfoPublic, MatchStatePhaseSpecifics, MatchStatePhaseSpecificsBetting, MatchStatePhaseSpecificsDrawing, DrawingStageDiscarding, HandCard};

use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::lobby::Lobby;
use crate::player::{Player, PlayerRegistry};
use crate::r#match::Match;
use structs::*;

impl GameState {
    pub fn build(
        lobby_state: LobbyState,
        match_state: Option<MatchState>,
    ) -> Self {
        GameState {
            lobby_state,
            match_state,
            timestamp: Utc::now(),
        }
    }

    pub fn as_player(&self, player_id: Uuid) -> Result<GameStateAsPlayer, AppError> {
        let lobby_state = self.lobby_state.clone();
        let mut own_match_state = None;
        if let Some(match_state) = self.match_state.as_ref() {
            let is_showdown = match match_state.poker_phase_specifics {
                MatchStatePhaseSpecifics::Showdown(_) => true,
                _ => false,
            };
            own_match_state = Some(match_state.as_player(&player_id, is_showdown)?.clone())
        };

        let state = GameStateAsPlayer {
            self_player_id: player_id,
            lobby_state,
            match_state: own_match_state,
            timestamp: self.timestamp,
        };
        Ok(state)
    }
}

impl LobbyState {
    pub async fn from_lobby(lobby: Lobby, player_registry_arc: Arc<RwLock<PlayerRegistry>>) -> Result<LobbyState, AppError> {
        let players = Self::get_players(&lobby, player_registry_arc).await?;
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
        Ok(
            LobbyState {
                lobby_id: lobby.lobby_id,
                name: lobby.name,
                host_player_id: lobby.host_player_id,
                players,
                status,
                game_acceptance,
                settings: lobby.settings,
            }
        )
    }

    async fn get_players(lobby: &Lobby, player_registry_arc: Arc<RwLock<PlayerRegistry>>) -> Result<Vec<PlayerPublicInfo>, AppError> {
        let player_ids = lobby.player_ids.clone();
        let player_registry_r = player_registry_arc.read().await;
        let players = player_registry_r.get_players(&player_ids).await?;
        let players = players
            .into_values()
            .map(|player| player.into())
            .collect();
        Ok(players)
    }
}

impl MatchState {
    pub fn as_player(&self, player_id: &Uuid, is_showdown: bool) -> Result<MatchStateAsPlayer, AppError> {
        let player_info = self.player_info
            .values()
            .map(|player_info| (player_info.player_id, player_info.as_player(player_id, !is_showdown)))
            .collect();

        let poker_phase_specifics = self.poker_phase_specifics
            .as_player(player_id)?;

        let can_player_act = self.can_player_act
            .get(&player_id)
            .cloned()
            .ok_or(AppError::internal("Invalid state [DEBUG]"))?; // TODO

        let state = MatchStateAsPlayer {
            match_id: self.match_id,
            player_info,
            credit_pots: self.credit_pots.clone(),
            player_bet_amounts: self.player_bet_amounts.clone(),
            poker_phase_specifics,
            can_player_act
        };
        Ok(state)
    }
}

impl MatchStatePhaseSpecifics {
    fn as_player(&self, player_id: &Uuid) -> Result<MatchStatePhaseSpecificsAsPlayer, AppError> {
        match self {
            MatchStatePhaseSpecifics::Ante => Ok(MatchStatePhaseSpecificsAsPlayer::Ante),
            MatchStatePhaseSpecifics::Dealing => Ok(MatchStatePhaseSpecificsAsPlayer::Dealing),
            MatchStatePhaseSpecifics::FirstBetting(phase) => {
                let self_bet_amount = phase.player_bet_amounts
                    .get(player_id)
                    .ok_or(AppError::internal(format!("Player ({player_id}) missing")))?
                    .clone();
                Ok(MatchStatePhaseSpecificsAsPlayer::FirstBetting(
                    MatchStatePhaseSpecificsBettingAsPlayer {
                        highest_bet_amount: phase.highest_bet_amount,
                        self_bet_amount,
                    }
                ))
            },
            MatchStatePhaseSpecifics::Drawing(phase) => {
                let phase_as_player = match phase {
                    MatchStatePhaseSpecificsDrawing::Discarding(discard_phase) => {
                        MatchStatePhaseSpecificsDrawingAsPlayer::Discarding(discard_phase.clone())
                    },
                    MatchStatePhaseSpecificsDrawing::Dealing => {
                        MatchStatePhaseSpecificsDrawingAsPlayer::Dealing
                    },
                };
                Ok(MatchStatePhaseSpecificsAsPlayer::Drawing(phase_as_player))
            },
            MatchStatePhaseSpecifics::SecondBetting(phase) => {
                let self_bet_amount = phase.player_bet_amounts
                    .get(player_id)
                    .ok_or(AppError::internal(format!("Player ({player_id}) missing")))?
                    .clone();
                Ok(MatchStatePhaseSpecificsAsPlayer::SecondBetting(
                    MatchStatePhaseSpecificsBettingAsPlayer {
                        highest_bet_amount: phase.highest_bet_amount,
                        self_bet_amount,
                    }
                ))
            },
            MatchStatePhaseSpecifics::Showdown(phase) => {
                Ok(MatchStatePhaseSpecificsAsPlayer::Showdown(
                    MatchStatePhaseSpecificsShowdownAsPlayer {
                        winning_rank: phase.winning_rank.clone(),
                        winner_ids: phase.winner_ids.clone(),
                        pot_distribution: phase.pot_distribution.clone(),
                    }
                ))
            },
        }
    }
}

impl GamePlayerPublicInfo {
    pub async fn from_match(r#match: &Match) -> HashMap<Uuid, Self> {
        let mut game_phase_w = r#match.phase.write().await;
        let player_credits = game_phase_w
            .get_table()
            .player_credits
            .clone();

        let player_cards = &game_phase_w
            .get_player_cards();

        player_credits
            .into_iter()
            .map(|(player_id, self_credits)| {
                let hand_cards = player_cards
                    .as_ref()
                    .and_then(|map| map.get(&player_id))
                    .cloned()
                    .flatten();
                let info = GamePlayerPublicInfo {
                    player_id: player_id.clone(),
                    player_name: "Anonymous".to_string(), // TODO
                    credits: self_credits,
                    hand_cards,
                };
                (player_id, info)
            })
            .collect()
    }

    pub fn as_player(
        &self,
        player_id: &Uuid,
        mask_foreign_cards: bool,
    ) -> GamePlayerPublicInfoAsPlayer {
        let mut player_info = self.clone();

        let show_card = *player_id == player_info.player_id || !mask_foreign_cards;
        let hand_cards = match player_info.hand_cards {
            None => None,
            Some(cards) => {
                Some(cards
                    .into_iter()
                    .map(|card| match (card.discarded, show_card) {
                        (true, _) => HandCard::DiscardedCard,
                        (false, true) => HandCard::VisibleCard(card.card),
                        (false, false) => HandCard::HiddenCard,
                    })
                    .collect())
            }
        };

        GamePlayerPublicInfoAsPlayer {
            player_id: player_info.player_id,
            player_name: player_info.player_name,
            credits: player_info.credits,
            hand_cards,
        }
    }
}

impl From<Player> for PlayerPublicInfo {
    fn from(player: Player) -> Self {
        PlayerPublicInfo {
            player_id: player.player_id,
            player_name:  player.player_name,
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

impl MatchState {
    pub(crate) async fn from_match(r#match: Match) -> Self {
        let player_info = GamePlayerPublicInfo::from_match(&r#match).await;
        let mut game_phase_w = r#match.phase.write().await;
        let player_cards = game_phase_w.get_player_cards();
        let credit_pots = game_phase_w.get_table().credit_pots.clone();
        let player_bet_amounts = game_phase_w.get_player_bet_amounts();
        let poker_phase_specifics = game_phase_w.get_phase_specifics();
        let can_player_act = game_phase_w.can_player_act();

        MatchState {
            match_id: r#match.match_id,
            player_info,
            credit_pots,
            player_bet_amounts,
            poker_phase_specifics,
            can_player_act,
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
