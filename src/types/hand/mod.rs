mod rank;
pub(crate) use rank::*;

use std::cmp::Ordering;
use thiserror::Error;
use crate::types::{
    card::{Card, CardRank, ShiftAce, GroupByRank},
};

#[derive(Eq, PartialEq, Debug, Hash)]
pub(crate) struct Hand {
    raw_hand_str: String, // eg: "AD KD QD JD 10D"
    cards: [Card; 5],
    rank: HandRank,
    tie_breakers: Option<TieBreakers>,
}

/// Tuple structs containing tie-breaker card ranks and vectors of card ranks.
/// Vector elements are sorted in descending order of priority.
#[derive(Eq, PartialEq, Debug, Hash)]
enum TieBreakers {
    // Kickers, Pairs and Sets for each HandRank
    StraightFlush(CardRank),     // top card
    FourOfAKind(Vec<CardRank>),  // quads rank, kicker
    FullHouse(Vec<CardRank>),    // trips rank, pair
    Flush(Vec<CardRank>),        // 5 kickers
    Straight(CardRank),          // top card
    ThreeOfAKind(Vec<CardRank>), // trips, high kicker, low kicker
    TwoPair(Vec<CardRank>),      // high pair, low pair, kicker
    Pair(Vec<CardRank>),         // pair, high kicker, mid kicker, low kicker
    HighCard(Vec<CardRank>),     // 5 kickers
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

impl PartialOrd for TieBreakers {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (TieBreakers::StraightFlush(self_rank), TieBreakers::StraightFlush(other_rank)) |
            (TieBreakers::Straight(self_rank), TieBreakers::Straight(other_rank)) => {
                Some(Ord::cmp(&self_rank, &other_rank))
            },
            (TieBreakers::FourOfAKind(self_ranks), TieBreakers::FourOfAKind(other_ranks)) |
            (TieBreakers::FullHouse(self_ranks), TieBreakers::FullHouse(other_ranks)) |
            (TieBreakers::Flush(self_ranks), TieBreakers::Flush(other_ranks)) |
            (TieBreakers::ThreeOfAKind(self_ranks), TieBreakers::ThreeOfAKind(other_ranks)) |
            (TieBreakers::TwoPair(self_ranks), TieBreakers::TwoPair(other_ranks)) |
            (TieBreakers::Pair(self_ranks), TieBreakers::Pair(other_ranks)) |
            (TieBreakers::HighCard(self_ranks), TieBreakers::HighCard(other_ranks)) => {
                let mut rank_pairs = self_ranks.iter().zip(other_ranks.iter());
                let ord = rank_pairs.find_map(|(self_rank, other_rank)| {
                    match Ord::cmp(&self_rank, &other_rank) {
                        Ordering::Equal => None, // keep going
                        ordering => Some(ordering),
                    }
                });
                Some(ord.unwrap_or(Ordering::Equal))
            },
            _ => None, // can't attempt tie-breaking between different hand ranks
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tie_breakers_partial_cmp() {
        assert!(
            TieBreakers::StraightFlush(CardRank::Ace)
                .partial_cmp(&TieBreakers::StraightFlush(CardRank::Ten))
                .is_some_and(|o| o == Ordering::Greater)
        );
        assert!(
            TieBreakers::FourOfAKind(vec![CardRank::Eight, CardRank::Four])
                .partial_cmp(&TieBreakers::FourOfAKind(vec![CardRank::Eight, CardRank::Ace]))
                .is_some_and(|o| o == Ordering::Less)
        );
        assert!(
            TieBreakers::FourOfAKind(vec![CardRank::Eight, CardRank::Ten])
                .partial_cmp(&TieBreakers::FourOfAKind(vec![CardRank::Eight, CardRank::Ten]))
                .is_some_and(|o| o == Ordering::Equal)
        );
        assert!(
            TieBreakers::StraightFlush(CardRank::Ace)
                .partial_cmp(&TieBreakers::FourOfAKind(vec![CardRank::Eight, CardRank::Ten]))
                .is_none()
        );
    }
}
