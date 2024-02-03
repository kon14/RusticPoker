use std::cmp::Ordering;
use crate::types::card::CardRank;

/// Tuple structs containing tie-breaker card ranks and vectors of card ranks.
/// Vector elements are sorted in descending order of priority.
#[derive(Eq, PartialEq, Debug, Hash)]
pub(crate) enum TieBreakers {
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
    fn partial_cmp() {
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
