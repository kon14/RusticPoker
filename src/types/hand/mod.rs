mod rank;
pub(crate) use rank::*;

use thiserror::Error;
use crate::types::{
    card::{Card, CardRank},
};

#[derive(Eq, PartialEq, Debug, Hash)]
pub(crate) struct Hand {
    raw_hand_str: String, // eg: "AD KD QD JD 10D"
    cards: [Card; 5],
    rank: HandRank,
    // tie_breakers: Option<TieBreakers>, // TODO
}

#[derive(Eq, PartialEq, Debug, Hash)]
enum TieBreakers {
    // Kickers, Pairs and Sets for each HandRank
    StraightFlush(CardRank),
    FourOfAKind(CardRank, CardRank),
    FullHouse(CardRank, CardRank),
    Flush([CardRank; 5]),
    Straight(CardRank),
    ThreeOfAKind(CardRank, [CardRank; 2]),
    TwoPair([CardRank; 2], CardRank),
    Pair(CardRank, [CardRank; 3]),
    HighCard([CardRank; 5]),
}

#[derive(Error, Debug)]
pub(crate) enum HandParseError {
    #[error("Invalid hand cards")]
    InvalidCards,
    #[error("Invalid hand length: `{0}`")]
    InvalidLength(usize),
}

impl TryFrom<&str> for Hand {
    type Error = HandParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let card_strs: Vec<&str> = value.split_whitespace().collect();
        if card_strs.len() != 5 {
            return Err(HandParseError::InvalidLength(card_strs.len()));
        }
        let cards = card_strs
            .into_iter()
            .map(|c| c.try_into().map_err(|err| HandParseError::InvalidCards))
            .collect::<Result<Vec<Card>, Self::Error>>()?;
        let cards: [Card; 5] = match cards.try_into() {
            Ok(array) => array,
            Err(_) => unreachable!(),
        };
        let rank: HandRank = cards.clone().into();

        // TODO: Tie Breakers

        Ok(Self {
            raw_hand_str: value.into(),
            cards,
            rank,
        })
    }
}

// TODO: Implement Hand equality, ordering
