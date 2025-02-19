use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::lobby::LobbySettings;
use crate::types::hand::Hand;
use crate::game::table::{CalculatedPlayerCredits, CreditPot};
use crate::types::card::Card;

#[derive(Clone, Debug)]
pub(crate) struct GameState {
    pub(super) player_states: HashMap<Uuid, PlayerState>,
    pub(super) lobby_state: LobbyState,
    pub(super) match_state: Option<MatchState>,
    pub(super) timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub(crate) struct GameStateAsPlayer {
    pub(super) player_state: PlayerState,
    pub(super) lobby_state: LobbyState,
    pub(super) match_state: Option<MatchStateAsPlayer>,
    pub(super) timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub(super) struct PlayerState {
    pub(super) player_id: Uuid,
    pub(super) name: String,
}

#[derive(Clone, Debug)]
pub(crate) enum LobbyStatus {
    Idle,
    Matchmaking,
    InGame,
}

#[derive(Clone, Debug)]
pub(super) struct LobbyState {
    pub(super) lobby_id: Uuid,
    pub(super) name: String,
    pub(super) host_player_id: Uuid,
    pub(super) player_ids: HashSet<Uuid>,
    pub(super) status: LobbyStatus,
    pub(super) game_acceptance: HashMap<Uuid, bool>,
    pub(super) settings: LobbySettings,
}

#[derive(Clone, Debug)]
pub(super) struct MatchState {
    pub(super) match_id: Uuid,
    pub(super) player_info: HashMap<Uuid, GamePlayerPublicInfo>,
    pub(super) player_cards: Option<HashMap<Uuid, Option<Vec<Card>>>>,
    pub(super) credit_pots: HashMap<Uuid, CreditPot>,
    pub(super) player_bet_amounts: Option<HashMap<Uuid, u64>>,
}

#[derive(Clone, Debug)]
pub(super) struct MatchStateAsPlayer {
    pub(super) match_id: Uuid,
    pub(super) player_info: HashMap<Uuid, GamePlayerPublicInfo>,
    pub(super) player_cards: Option<Vec<Card>>,
    pub(super) credit_pots: HashMap<Uuid, CreditPot>,
    pub(super) player_bet_amounts: Option<HashMap<Uuid, u64>>,
}

#[derive(Clone, Debug)]
pub(super) struct GamePlayerPublicInfo {
    pub(super) player_id: Uuid,
    // pub(super) name: String,
    pub(super) credits: CalculatedPlayerCredits,
    pub(super) hand_card_count: u8,
}

pub(crate) struct LobbyInfoPublic {
    pub(super) lobby_id: Uuid,
    pub(super) name: String,
    pub(super) host_player_id: Uuid,
    pub(super) player_count: u32,
    pub(super) status: LobbyStatus,
}
