use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::lobby::LobbySettings;
use crate::types::hand::HandRank;
use crate::game::table::{CalculatedPlayerCredits, CreditPot};
use crate::types::card::Card;
use crate::types::stateful::StatefulCard;

#[derive(Clone, Debug)]
pub(crate) struct GameState {
    pub(super) lobby_state: LobbyState,
    pub(super) match_state: Option<MatchState>,
    pub(super) timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub(crate) struct GameStateAsPlayer {
    pub(super) self_player_id: Uuid,
    pub(super) lobby_state: LobbyState,
    pub(super) match_state: Option<MatchStateAsPlayer>,
    pub(super) timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub(super) struct PlayerPublicInfo {
    pub(super) player_id: Uuid,
    pub(super) player_name: String,
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
    pub(super) players: Vec<PlayerPublicInfo>,
    pub(super) status: LobbyStatus,
    pub(super) game_acceptance: HashMap<Uuid, bool>,
    pub(super) settings: LobbySettings,
}

#[derive(Clone, Debug)]
pub(super) struct MatchState {
    pub(super) match_id: Uuid,
    pub(super) player_info: HashMap<Uuid, GamePlayerPublicInfo>,
    pub(super) credit_pots: HashMap<Uuid, CreditPot>,
    pub(super) player_bet_amounts: Option<HashMap<Uuid, u64>>,
    pub(super) poker_phase_specifics: MatchStatePhaseSpecifics,
    pub(super) can_player_act: HashMap<Uuid, bool>,
}

#[derive(Clone, Debug)]
pub(super) struct MatchStateAsPlayer {
    pub(super) match_id: Uuid,
    pub(super) player_info: HashMap<Uuid, GamePlayerPublicInfoAsPlayer>,
    pub(super) credit_pots: HashMap<Uuid, CreditPot>,
    pub(super) player_bet_amounts: Option<HashMap<Uuid, u64>>,
    pub(super) poker_phase_specifics: MatchStatePhaseSpecificsAsPlayer,
    pub(super) can_player_act: bool,
}

#[derive(Clone, Debug)]
pub(super) struct GamePlayerPublicInfo {
    pub(super) player_id: Uuid,
    pub(super) player_name: String,
    pub(super) credits: CalculatedPlayerCredits,
    pub(super) hand_cards: Option<Vec<StatefulCard>>,
}

#[derive(Clone, Debug)]
pub(super) struct GamePlayerPublicInfoAsPlayer {
    pub(super) player_id: Uuid,
    pub(super) player_name: String,
    pub(super) credits: CalculatedPlayerCredits,
    pub(super) hand_cards: Option<Vec<HandCard>>,
}

pub(crate) struct LobbyInfoPublic {
    pub(super) lobby_id: Uuid,
    pub(super) name: String,
    pub(super) host_player: PlayerPublicInfo,
    pub(super) player_count: u32,
    pub(super) status: LobbyStatus,
    pub(super) settings: LobbySettings,
    pub(super) is_joinable: bool,
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
pub(crate) enum MatchStatePhaseSpecificsDrawing {
    Discarding(DrawingStageDiscarding),
    Dealing,
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
    pub(super) self_bet_amount: u64,
}

#[derive(Clone, Debug)]
pub(crate) enum MatchStatePhaseSpecificsDrawingAsPlayer {
    Discarding(DrawingStageDiscarding),
    Dealing,
}

#[derive(Clone, Debug)]
pub(crate) struct MatchStatePhaseSpecificsShowdownAsPlayer {
    pub(crate) winning_rank: HandRank,
    pub(crate) winner_ids: HashSet<Uuid>,
    pub(crate) pot_distribution: HashMap<Uuid, ShowdownPotDistribution>,
}

#[derive(Clone, Debug)]
pub struct DrawingStageDiscarding {
    pub(crate) player_discard_count: HashMap<Uuid, u8>,
}

#[derive(Clone, Debug)]
pub(crate) struct ShowdownPotDistribution {
    pub(crate) pot_id: Uuid,
    pub(crate) player_ids: HashSet<Uuid>,
    pub(crate) total_credits: u64,
    pub(crate) credits_per_winner: u64,
}

#[derive(Clone, Debug)]
pub(crate) enum HandCard {
    VisibleCard(Card),
    HiddenCard,
    DiscardedCard,
}
