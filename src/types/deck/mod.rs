use rand::{rng, seq::SliceRandom};

use super::card::{Card, CardRank, CardSuit};

#[derive(Clone, Debug)]
pub struct CardDeck(Vec<Card>);

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
            // let mut rank_iter = CardRank::Two.into_iter();
            // deck.push(Card { rank, suit: suit.clone() });
            // while let Some(rank) = rank_iter.next() {
            //     deck.push(Card { rank, suit: suit.clone() });
            // }
        }
        let mut deck = CardDeck(deck);
        deck.shuffle();
        deck
    }
}

impl CardDeck {
    pub fn draw(&mut self) -> Option<Card> {
        self.0.pop()
    }

    fn shuffle(&mut self) {
        let mut rng = rng();
        self.0.shuffle(&mut rng);
    }
}
