mod rank;
mod tie_breakers;

pub(crate) use rank::*;

use thiserror::Error;
use crate::types::{
    card::{Card, CardRank, ShiftAce, GroupByRank},
    hand::tie_breakers::TieBreakers,
};

#[derive(Eq, PartialEq, Debug, Hash)]
pub(crate) struct Hand {
    raw_hand_str: String, // eg: "AD KD QD JD 10D"
    cards: [Card; 5],
    rank: HandRank,
    tie_breakers: Option<TieBreakers>,
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
            .map(|c| c.try_into().map_err(|_| HandParseError::InvalidCards))
            .collect::<Result<Vec<Card>, Self::Error>>()?;
        let cards: [Card; 5] = match cards.try_into() {
            Ok(array) => array,
            Err(_) => unreachable!(),
        };
        let rank: HandRank = cards.clone().into();
        let tie_breakers = get_tie_breakers(&rank, &cards);
        return Ok(Self {
            raw_hand_str: value.into(),
            cards,
            rank,
            tie_breakers,
        });

        fn get_tie_breakers(rank: &HandRank, cards: &[Card; 5]) -> Option<TieBreakers> {
            match (rank, cards) {
                (&HandRank::RoyalFlush, _) => None,
                (rank @ (&HandRank::StraightFlush | &HandRank::Straight), cards) => {
                    let mut ranks_asc: Vec<CardRank> = cards.iter().map(|card| card.rank.clone()).collect();
                    if ranks_asc.contains(&CardRank::Ace) && ranks_asc.contains(&CardRank::Two) {
                        ranks_asc = ranks_asc.shift_ace().unwrap(); // asc-sorted
                    } else {
                        ranks_asc.sort();
                    }
                    let top_rank = ranks_asc.last().unwrap().clone();
                    match rank {
                        &HandRank::StraightFlush => Some(TieBreakers::StraightFlush(top_rank)),
                        _ => Some(TieBreakers::Straight(top_rank))
                    }
                },
                (&HandRank::FourOfAKind, cards) => {
                    let ranks: Vec<CardRank> = cards.iter().map(|card| card.rank.clone()).collect();
                    let mut groups = ranks.group_by_rank();
                    let (mut group_a, mut group_b) = (groups.pop().unwrap(), groups.pop().unwrap());
                    let (quads_rank, kicker_rank) = match (group_a.len(), group_b.len()) {
                        (4, 1) => (group_a.pop().unwrap(), group_b.pop().unwrap()),
                        _ => (group_b.pop().unwrap(), group_a.pop().unwrap())
                    };
                    Some(TieBreakers::FourOfAKind(vec![quads_rank, kicker_rank]))
                },
                (&HandRank::FullHouse, cards) => {
                    let ranks: Vec<CardRank> = cards.iter().map(|card| card.rank.clone()).collect();
                    let mut groups = ranks.group_by_rank();
                    let (mut group_a, mut group_b) = (groups.pop().unwrap(), groups.pop().unwrap());
                    let (trips_rank, pair_rank) = match (group_a.len(), group_b.len()) {
                        (3, 2) => (group_a.pop().unwrap(), group_b.pop().unwrap()),
                        _ => (group_b.pop().unwrap(), group_a.pop().unwrap())
                    };
                    Some(TieBreakers::FullHouse(vec![trips_rank, pair_rank]))
                },
                (rank @ (&HandRank::Flush | &HandRank::HighCard), cards) => {
                    let ranks_desc: Vec<CardRank> = cards.iter().map(|card| card.rank.clone()).collect();
                    let mut ranks_desc: [CardRank; 5] = ranks_desc.try_into().unwrap();
                    ranks_desc.sort_by(|a, b| Ord::cmp(b, a));
                    match rank {
                        &HandRank::Flush => Some(TieBreakers::Flush(ranks_desc.to_vec())),
                        _ => Some(TieBreakers::HighCard(ranks_desc.to_vec())),
                    }
                },
                (&HandRank::ThreeOfAKind, cards) => {
                    let ranks: Vec<CardRank> = cards.iter().map(|card| card.rank.clone()).collect();
                    let mut groups = ranks.group_by_rank();
                    groups.sort_by(|a, b| b.len().cmp(&a.len())); // desc-sorted by length
                    let kicker_a = groups.pop().unwrap()[0].clone();
                    let kicker_b = groups.pop().unwrap()[0].clone();
                    let trips_rank = groups.pop().unwrap()[0].clone();
                    if kicker_a > kicker_b {
                        Some(TieBreakers::ThreeOfAKind(vec![trips_rank, kicker_a, kicker_b]))
                    } else {
                        Some(TieBreakers::ThreeOfAKind(vec![trips_rank, kicker_b, kicker_a]))
                    }
                },
                (&HandRank::TwoPair, cards) => {
                    let ranks: Vec<CardRank> = cards.iter().map(|card| card.rank.clone()).collect();
                    let mut groups = ranks.group_by_rank();
                    groups.sort_by(|a, b| b.len().cmp(&a.len())); // desc-sorted by length
                    let kicker_rank = groups.pop().unwrap()[0].clone();
                    let pair_rank_a = groups.pop().unwrap()[0].clone();
                    let pair_rank_b = groups.pop().unwrap()[0].clone();
                    if pair_rank_a > pair_rank_b {
                        Some(TieBreakers::TwoPair(vec![pair_rank_a, pair_rank_b, kicker_rank]))
                    } else {
                        Some(TieBreakers::TwoPair(vec![pair_rank_b, pair_rank_a, kicker_rank]))
                    }
                },
                (&HandRank::Pair, cards) => {
                    let ranks: Vec<CardRank> = cards.iter().map(|card| card.rank.clone()).collect();
                    let mut groups = ranks.group_by_rank();
                    groups.sort_by(|a, b| b.len().cmp(&a.len())); // asc-sorted by length
                    let pair_rank = groups.pop().unwrap()[0].clone();
                    let rank_a = groups.pop().unwrap()[0].clone();
                    let rank_b = groups.pop().unwrap()[0].clone();
                    let rank_c = groups.pop().unwrap()[0].clone();
                    let mut kickers = [rank_a, rank_b, rank_c];
                    kickers.sort_by(|a, b| Ord::cmp(b, a)); // desc-sorted
                    let [high_kicker, mid_kicker, low_kicker] = kickers;
                    Some(TieBreakers::Pair(vec![pair_rank, high_kicker, mid_kicker, low_kicker]))
                },
            }
        }
    }
}
