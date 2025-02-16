use std::collections::HashSet;
use rand::{rng, seq::SliceRandom};

use super::card::{Card, CardRank, CardSuit};

#[derive(Clone, Debug)]
pub struct CardDeck {
    deck: Vec<Card>,
    discard_pile: HashSet<Card>,
}

impl Default for CardDeck {
    fn default() -> Self {
        let mut deck = Vec::with_capacity(52);
        let suits = [CardSuit::Diamonds, CardSuit::Hearts, CardSuit::Clubs, CardSuit::Spades];
        for suit in suits {
            deck.push(Card { rank: CardRank::Two, suit: suit.clone() });
            deck.extend(
                CardRank::Two
                    .into_iter()
                    .map(|rank| Card { rank, suit: suit.clone() })
            );
        }
        let mut deck = CardDeck {
            deck,
            discard_pile: HashSet::default(),
        };
        deck.shuffle();
        deck
    }
}

impl CardDeck {
    pub fn draw(&mut self) -> Option<Card> {
        self.deck.pop()
    }

    fn shuffle(&mut self) {
        let mut rng = rng();
        self.deck.shuffle(&mut rng);
    }

    pub fn discard_cards(&mut self, cards: HashSet<Card>) {
        self.discard_pile.extend(cards);
    }
}
