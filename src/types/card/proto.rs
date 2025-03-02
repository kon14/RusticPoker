use crate::common::error::AppError;
use crate::service::proto;
use super::{Card, CardRank, CardSuit};

impl From<CardRank> for proto::card::CardRank {
    fn from(rank: CardRank) -> Self {
        match rank {
            CardRank::Ace => proto::card::CardRank::Ace,
            CardRank::Two => proto::card::CardRank::Two,
            CardRank::Three => proto::card::CardRank::Three,
            CardRank::Four => proto::card::CardRank::Four,
            CardRank::Five => proto::card::CardRank::Five,
            CardRank::Six => proto::card::CardRank::Six,
            CardRank::Seven => proto::card::CardRank::Seven,
            CardRank::Eight => proto::card::CardRank::Eight,
            CardRank::Nine => proto::card::CardRank::Nine,
            CardRank::Ten => proto::card::CardRank::Ten,
            CardRank::Jack => proto::card::CardRank::Jack,
            CardRank::Queen => proto::card::CardRank::Queen,
            CardRank::King => proto::card::CardRank::King,
        }
    }
}

impl From<proto::card::CardRank> for CardRank {
    fn from(rank: proto::card::CardRank) -> Self {
        match rank {
            proto::card::CardRank::Ace => CardRank::Ace,
            proto::card::CardRank::Two => CardRank::Two,
            proto::card::CardRank::Three => CardRank::Three,
            proto::card::CardRank::Four => CardRank::Four,
            proto::card::CardRank::Five => CardRank::Five,
            proto::card::CardRank::Six => CardRank::Six,
            proto::card::CardRank::Seven => CardRank::Seven,
            proto::card::CardRank::Eight => CardRank::Eight,
            proto::card::CardRank::Nine => CardRank::Nine,
            proto::card::CardRank::Ten => CardRank::Ten,
            proto::card::CardRank::Jack => CardRank::Jack,
            proto::card::CardRank::Queen => CardRank::Queen,
            proto::card::CardRank::King => CardRank::King,
        }
    }
}

impl From<CardSuit> for proto::card::CardSuit {
    fn from(suit: CardSuit) -> Self {
        match suit {
            CardSuit::Diamonds => proto::card::CardSuit::Diamonds,
            CardSuit::Hearts => proto::card::CardSuit::Hearts,
            CardSuit::Clubs => proto::card::CardSuit::Clubs,
            CardSuit::Spades => proto::card::CardSuit::Spades,
        }
    }
}

impl From<proto::card::CardSuit> for CardSuit {
    fn from(suit: proto::card::CardSuit) -> Self {
        match suit {
            proto::card::CardSuit::Diamonds => CardSuit::Diamonds,
            proto::card::CardSuit::Hearts => CardSuit::Hearts,
            proto::card::CardSuit::Clubs => CardSuit::Clubs,
            proto::card::CardSuit::Spades => CardSuit::Spades,
        }
    }
}

impl From<Card> for proto::Card {
    fn from(card: Card) -> Self {
        let rank: proto::card::CardRank = card.rank.into();
        let suit: proto::card::CardSuit = card.suit.into();
        proto::Card {
            rank: rank as i32,
            suit: suit as i32,
        }
    }
}

impl TryFrom<proto::Card> for Card {
    type Error = AppError;

    fn try_from(card: proto::Card) -> Result<Self, Self::Error> {
        let rank = proto::card::CardRank::try_from(card.rank)
            .map_err(|_| AppError::invalid_request(format!("Invalid card rank: {}!", card.rank)))?
            .into();
        let suit = proto::card::CardSuit::try_from(card.suit)
            .map_err(|_| AppError::invalid_request(format!("Invalid card suit: {}!", card.suit)))?
            .into();
        Ok(Card {
            rank,
            suit,
        })
    }
}

// impl<'a> TryFrom<&'a str> for Card {
//     type Error = CardParseError<'a>;
//     fn try_from(value: &'a str) -> Result<Self, Self::Error> {
//         let (rank, suit) = value.split_at(value.len() - 1);
//         let rank: CardRank = rank.try_into()?;
//         let suit: CardSuit = suit.try_into()?;
//         Ok(Card { rank, suit })
//     }
// }
//
// impl From<Card> for proto::Card {
//     fn from(card: Card) -> Self {
//         proto::Card {
//             card: card.into(),
//         }
//     }
// }
