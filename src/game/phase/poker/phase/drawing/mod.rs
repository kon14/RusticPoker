mod dealing;
mod discarding;

use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::Deref;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::game::GameTable;
use crate::service::proto;
use crate::types::card::Card;
use crate::types::deck::CardDeck;
use crate::types::hand::Hand;

#[derive(Clone, Debug)]
pub(crate) struct PokerPhaseDrawingDealing {
    pub(crate) rpc_action_broadcaster: broadcast::Sender<()>,
    pub(crate) game_table: GameTable,
    pub(crate) card_deck: CardDeck,
    pub(crate) phase_player_queue: VecDeque<Uuid>,
    pub(crate) _player_bets: HashMap<Uuid, u64>,
    pub(crate) player_hands: HashMap<Uuid, Hand>, // folded hands omitted
    pub(crate) player_discarded_cards: HashMap<Uuid, Option<DiscardedCards>>, // TODO: display (output) own discarded cards, foreign discarded count
}

#[derive(Clone, Debug)]
pub(crate) struct PokerPhaseDrawingDiscarding {
    pub(crate) rpc_action_broadcaster: broadcast::Sender<()>,
    pub(crate) game_table: GameTable,
    pub(crate) card_deck: CardDeck,
    pub(super) phase_player_queue: VecDeque<Uuid>,
    pub(crate) _player_bets: HashMap<Uuid, u64>,
    pub(crate) player_hands: HashMap<Uuid, Hand>, // folded hands omitted
    pub(crate) player_discarded_cards: HashMap<Uuid, Option<DiscardedCards>>, // TODO: display (output) own discarded cards, foreign discarded count
}

#[derive(Clone, Debug)]
pub(crate) struct DiscardedCards(HashSet<Card>);

impl DiscardedCards {
    pub(crate) fn try_from_proto(request: proto::RespondDrawingPhaseRequest) -> Result<Option<Self>, AppError> {
        let proto::RespondDrawingPhaseRequest { discarded_cards } = request;
        if discarded_cards.is_empty() {
            Ok(None)
        } else {
            let discarded_cards = discarded_cards
                .into_iter()
                .map(|card| card.try_into())
                .collect::<Result<_, _>>()?;
            Ok(Some(DiscardedCards(discarded_cards)))
        }
    }
}

impl Deref for DiscardedCards {
    type Target = HashSet<Card>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
