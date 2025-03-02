use std::collections::HashMap;
use uuid::Uuid;

use crate::game::DiscardedCards;
use crate::types::card::Card;
use crate::types::hand::Hand;

// TODO: Consider using StatefulHand, refactor Drawing phase discard source of truth
// #[derive(Clone, Debug)]
// pub(crate) struct StatefulHand {
//     pub(crate) player_id: Uuid,
//     pub(crate) raw_hand_str: String, // eg: "AD KD QD JD 10D"
//     pub(crate) cards: [StatefulCard; 5],
//     pub(crate) rank: HandRank,
//     pub(crate) tie_breakers: Option<TieBreakers>,
// }

#[derive(Clone, Debug)]
pub(crate) struct StatefulCard {
    pub(crate) card: Card,
    pub(crate) discarded: bool,
}

impl From<Card> for StatefulCard {
    fn from(card: Card) -> Self {
        StatefulCard {
            card,
            discarded: false,
        }
    }
}

pub(crate) fn get_stateful_cards(
    player_hands: &HashMap<Uuid, Hand>,
    player_discarded_cards: Option<&HashMap<Uuid, Option<DiscardedCards>>>,
) -> HashMap<Uuid, Option<Vec<StatefulCard>>> {
    player_hands
        .iter()
        .map(|(player_id, hand)| {
            let mut stateful_cards: Vec<StatefulCard> = hand.cards
                .iter()
                .map(|card| card.clone().into())
                .collect();
            let Some(player_discarded_cards) = player_discarded_cards else {
                return (player_id.clone(), Some(stateful_cards));
            };
            let Some(player_discarded_cards) = player_discarded_cards.get(player_id) else {
                return (player_id.clone(), Some(stateful_cards));
            };
            let Some(player_discarded_cards) = player_discarded_cards else {
                return (player_id.clone(), Some(stateful_cards));
            };
            stateful_cards
                .iter_mut()
                .for_each(|card| {
                    if player_discarded_cards.contains(&card.card) {
                        card.discarded = true;
                    }
                });
            (player_id.clone(), Some(stateful_cards))
        })
        .collect()
}
