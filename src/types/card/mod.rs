mod proto;
mod rank;
mod suit;

pub(crate) use rank::*;
pub(crate) use suit::*;

use std::cmp::Ordering;
use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use thiserror::Error;

use crate::common::error::AppError;

#[derive(Eq, Clone, Debug, Hash)]
pub(crate) struct Card {
    pub rank: CardRank,
    pub suit: CardSuit,
}

#[derive(Error, Debug)]
pub(crate) enum CardParseError<'a> {
    #[error("Invalid card rank: `{0}`")]
    InvalidRank(&'a str),
    #[error("Invalid card suit: `{0}`")]
    InvalidSuit(&'a str),
}

impl From<CardParseError<'_>> for AppError {
    fn from(error: CardParseError) -> Self {
        let card_str = match error {
            CardParseError::InvalidRank(card_str) => card_str,
            CardParseError::InvalidSuit(card_str) => card_str,
        };
        AppError::InvalidRequest(format!("Invalid card representation: {card_str}!"))
    }
}

impl<'a> TryFrom<&'a str> for Card {
    type Error = CardParseError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (rank, suit) = value.split_at(value.len() - 1);
        let rank: CardRank = rank.try_into()?;
        let suit: CardSuit = suit.try_into()?;
        Ok(Card { rank, suit })
    }
}

impl PartialEq for Card {
    // "AH" == "AC"
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.rank, &other.rank)
    }
}

impl PartialOrd<Self> for Card {
    // https://github.com/rust-lang/rust/issues/63104
    // https://github.com/rust-lang/rfcs/pull/1028
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}
