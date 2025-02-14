mod betting;
mod drawing;

use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::Deref;
use uuid::Uuid;

use crate::game::GameTable;
use crate::types::card::Card;
use crate::types::deck::CardDeck;
use crate::types::hand::Hand;

#[derive(Clone, Debug)]
pub(super) struct PokerPhaseBetting {
    pub(super) game_table: GameTable,
    pub(super) card_deck: CardDeck,
    pub(super) phase_player_queue: VecDeque<Uuid>,
    pub(super) player_hands: HashMap<Uuid, Hand>, // folded hands omitted
    pub(super) player_bets: HashMap<Uuid, u64>, // folded players omitted

    // TODO: display current player (here or in wrapper struct)
    //       maybe use HashSet<Uuid> ? that way unordered round phases can omit past players...
}

#[derive(Clone, Debug)]
pub(super) struct PokerPhaseAnte {
    pub(super) game_table: GameTable,
    pub(super) card_deck: CardDeck,
    pub(super) phase_player_queue: VecDeque<Uuid>,
    pub(super) ante_amount: u64,
}

#[derive(Clone, Debug)]
pub(super) struct PokerPhaseDealing {
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
pub(super) struct PokerPhaseDrawing {
    pub(super) game_table: GameTable,
    pub(super) card_deck: CardDeck,
    pub(super) phase_player_queue: VecDeque<Uuid>,
    pub(super) _player_bets: HashMap<Uuid, u64>,
    pub(super) player_hands: HashMap<Uuid, Hand>, // folded hands omitted
    pub(super) player_discarded_cards: HashMap<Uuid, Option<HashSet<Card>>>,
}

#[derive(Clone, Debug)]
pub(super) struct PokerPhaseSecondBetting(pub(super) PokerPhaseBetting);

#[derive(Clone, Debug)]
pub(super) struct PokerPhaseShowdown {
    pub(super) game_table: GameTable,
    pub(super) card_deck: CardDeck,
    pub(super) phase_player_queue: VecDeque<Uuid>,
    pub(super) player_hands: HashMap<Uuid, Hand>, // folded hands omitted, all hands revealed at the same time
}

impl Deref for PokerPhaseFirstBetting {
    type Target = PokerPhaseBetting;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for PokerPhaseSecondBetting {
    type Target = PokerPhaseBetting;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<PokerPhaseBetting> for PokerPhaseFirstBetting {
    fn as_ref(&self) -> &PokerPhaseBetting {
        &self.0
    }
}

impl AsRef<PokerPhaseBetting> for PokerPhaseSecondBetting {
    fn as_ref(&self) -> &PokerPhaseBetting {
        &self.0
    }
}
