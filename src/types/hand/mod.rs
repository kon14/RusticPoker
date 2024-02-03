mod rank;
mod tie_breakers;

pub(crate) use rank::*;

use std::cmp::Ordering;
use thiserror::Error;
use crate::types::{
    card::{Card, CardRank, ShiftAce, GroupByRank},
    hand::tie_breakers::TieBreakers,
};

#[derive(Eq, Clone, Debug, Hash)]
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

impl PartialEq for Hand {
    // "AS KS QS JS 10S" == "AH KH QH JH 10H"
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.rank != other.rank || (self.tie_breakers.is_none()) {
            // No tie-breaking for non-equal hand ranks or RoyalFlush
            self.rank.cmp(&other.rank)
        } else {
            self.tie_breakers.partial_cmp(&other.tie_breakers).unwrap()
        }
    }
}

impl PartialOrd<Self> for Hand {
    // https://github.com/rust-lang/rust/issues/63104
    // https://github.com/rust-lang/rfcs/pull/1028
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;
    #[test]
    fn cmp() {
        let royal_flush: Hand = "AS KS QS JS 10S".try_into().unwrap();
        let straight_flush: Hand = "9S 8S 7S 6S 5S".try_into().unwrap();
        let four_of_a_kind: Hand = "2H 2S 2D 2C 9S".try_into().unwrap();
        let full_house: Hand = "KH KD 5S 5D 5C".try_into().unwrap();
        let flush: Hand = "3D 7D 9D JD AD".try_into().unwrap();
        let straight: Hand = "8C 9D 10H JS QS".try_into().unwrap();
        let three_of_a_kind: Hand = "7H 7S 7D AC 2H" .try_into().unwrap();
        let two_pair: Hand = "AH AD 3S 3H 6C".try_into().unwrap();
        let pair: Hand = "10S 10H 8D 7C 2S".try_into().unwrap();
        let high_card: Hand = "QD 9H 7C 5S 3H".try_into().unwrap();
        let hands_sorted = [
            royal_flush,
            straight_flush,
            four_of_a_kind,
            full_house,
            flush,
            straight,
            three_of_a_kind,
            two_pair.clone(),
            pair,
            high_card.clone(),
        ];
        let mut hands_rand = hands_sorted.clone();
        let mut rng = rand::thread_rng();
        hands_rand.shuffle(&mut rng);
        hands_rand.sort_by(|a, b| b.cmp(a));
        assert_eq!(hands_sorted, hands_rand);
        let royal_flush_s: Hand = "AS KS QS JS 10S".try_into().unwrap();
        let royal_flush_h: Hand = "AH KH QH JH 10H".try_into().unwrap();
        assert_eq!(royal_flush_s, royal_flush_h);
        let two_pair_high: Hand = "AH AD 3S 3H 6C".try_into().unwrap();
        let two_pair_low: Hand = "AH AD 2S 2H 9C".try_into().unwrap();
        assert_eq!(Ord::cmp(&two_pair_high, &two_pair_low), Ordering::Greater);
    }
}
