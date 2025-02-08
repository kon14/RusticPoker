use crate::service::proto;
use super::Hand;

impl From<Hand> for proto::Hand {
    fn from(hand: Hand) -> Self {
        let cards = hand.cards.map(|card| card.into()).into();
        proto::Hand {
            cards
        }
    }
}
