mod betting;
mod drawing;

pub(crate) use betting::BettingRoundAction;
pub(crate) use drawing::{PokerPhaseDrawingDiscarding, PokerPhaseDrawingDealing, DiscardedCards};

use std::collections::{HashMap, VecDeque};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::game::GameTable;
use crate::types::deck::CardDeck;
use crate::types::hand::Hand;

#[derive(Clone, Debug)]
pub(super) struct PokerPhaseBetting {
    pub(super) rpc_action_broadcaster: broadcast::Sender<()>,
    pub(super) game_table: GameTable,
    pub(super) card_deck: CardDeck,
    pub(super) phase_player_queue: VecDeque<Uuid>,
    pub(super) player_hands: HashMap<Uuid, Hand>, // folded hands omitted
    pub(super) player_bets: HashMap<Uuid, u64>, // folded players omitted
    pub(super) first_round_action: bool,

    // TODO: display current player (here or in wrapper struct)
    //       maybe use HashSet<Uuid> ? that way unordered round phases can omit past players...
}

#[derive(Clone, Debug)]
pub(super) struct PokerPhaseAnte {
    pub(super) _rpc_action_broadcaster: broadcast::Sender<()>,
    pub(super) game_table: GameTable,
    pub(super) card_deck: CardDeck,
    pub(super) phase_player_queue: VecDeque<Uuid>,
    pub(super) ante_amount: u64,
}

#[derive(Clone, Debug)]
pub(super) struct PokerPhaseDealing {
    pub(super) _rpc_action_broadcaster: broadcast::Sender<()>,
    pub(super) game_table: GameTable,
    pub(super) card_deck: CardDeck,
    pub(super) phase_player_queue: VecDeque<Uuid>,
    pub(super) _ante_amount: u64,
    pub(super) player_hands: HashMap<Uuid, Hand>, // folded hands omitted
    //pub(super) player_cards: HashMap<Uuid, HashSet<Card>>,
}

#[derive(Clone, Debug)]
pub(super) struct PokerPhaseFirstBetting(pub(super) PokerPhaseBetting);

#[derive(Clone, Debug)]
pub(super) struct PokerPhaseSecondBetting(pub(super) PokerPhaseBetting);

#[derive(Clone, Debug)]
pub(super) struct PokerPhaseShowdown {
    pub(super) game_table: GameTable,
    pub(super) card_deck: CardDeck,
    pub(super) phase_player_queue: VecDeque<Uuid>,
    pub(super) player_hands: HashMap<Uuid, Hand>, // folded hands omitted, all hands revealed at the same time
}
