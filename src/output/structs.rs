use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::lobby::LobbySettings;
use crate::types::hand::HandRank;
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
    pub(super) poker_phase_specifics: MatchStatePhaseSpecifics,
}

#[derive(Clone, Debug)]
pub(super) struct MatchStateAsPlayer {
    pub(super) match_id: Uuid,
    pub(super) player_info: HashMap<Uuid, GamePlayerPublicInfo>,
    pub(super) player_cards: Option<Vec<Card>>,
    pub(super) credit_pots: HashMap<Uuid, CreditPot>,
    pub(super) player_bet_amounts: Option<HashMap<Uuid, u64>>,
    pub(super) poker_phase_specifics: MatchStatePhaseSpecificsAsPlayer,
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

#[derive(Clone, Debug)]
pub(crate) enum MatchStatePhaseSpecifics {
    Ante,
    Dealing,
    FirstBetting(MatchStatePhaseSpecificsBetting),
    Drawing(MatchStatePhaseSpecificsDrawing),
    SecondBetting(MatchStatePhaseSpecificsBetting),
    Showdown(MatchStatePhaseSpecificsShowdown),
}

#[derive(Clone, Debug)]
pub(crate) struct MatchStatePhaseSpecificsBetting {
    pub(crate) highest_bet_amount: u64,
    pub(crate) player_bet_amounts: HashMap<Uuid, u64>, // I don't really need this. part of parent struct. could be convenient for mapping tho
}

#[derive(Clone, Debug)]
pub(crate) struct MatchStatePhaseSpecificsDrawing {
    pub(crate) discard_stage: bool,
    pub(crate) discarded_cards: HashMap<Uuid, HashSet<Card>>, // TODO: Vec if ordered
}

#[derive(Clone, Debug)]
pub(crate) struct MatchStatePhaseSpecificsShowdown {
    pub(crate) winning_rank: HandRank,
    pub(crate) winner_ids: HashSet<Uuid>,
    pub(crate) pot_distribution: HashMap<Uuid, ShowdownPotDistribution>,
}

#[derive(Clone, Debug)]
pub(super) enum MatchStatePhaseSpecificsAsPlayer {
    Ante,
    Dealing,
    FirstBetting(MatchStatePhaseSpecificsBettingAsPlayer),
    Drawing(MatchStatePhaseSpecificsDrawingAsPlayer),
    SecondBetting(MatchStatePhaseSpecificsBettingAsPlayer),
    Showdown(MatchStatePhaseSpecificsShowdownAsPlayer),
}

#[derive(Clone, Debug)]
pub(super) struct MatchStatePhaseSpecificsBettingAsPlayer {
    pub(super) highest_bet_amount: u64,
    pub(super) own_bet_amount: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct MatchStatePhaseSpecificsDrawingAsPlayer {
    pub(crate) discard_stage: bool,
    pub(crate) own_discarded_cards: HashSet<Card>, // TODO: Vec if ordered
}

#[derive(Clone, Debug)]
pub(crate) struct MatchStatePhaseSpecificsShowdownAsPlayer {
    pub(crate) winning_rank: HandRank,
    pub(crate) winner_ids: HashSet<Uuid>,
    pub(crate) pot_distribution: HashMap<Uuid, ShowdownPotDistribution>,
}

#[derive(Clone, Debug)]
pub(crate) struct ShowdownPotDistribution {
    pub(crate) pot_id: Uuid,
    pub(crate) player_ids: HashSet<Uuid>,
    pub(crate) total_credits: u64,
    pub(crate) credits_per_winner: u64,
}
