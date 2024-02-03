use std::cmp::Ordering;
use std::collections::HashMap;
use crate::types::card::{Card, CardRank, CardSuit, ShiftAce};

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub(crate) enum HandRank {
    RoyalFlush,
    StraightFlush,
    FourOfAKind,
    FullHouse,
    Flush,
    Straight,
    ThreeOfAKind,
    TwoPair,
    Pair,
    HighCard,
}

impl HandRank {
    const ORDER: [Self; 10] = [
        Self::RoyalFlush,
        Self::StraightFlush,
        Self::FourOfAKind,
        Self::FullHouse,
        Self::Flush,
        Self::Straight,
        Self::ThreeOfAKind,
        Self::TwoPair,
        Self::Pair,
        Self::HighCard,
    ];
}

impl From<[Card; 5]> for HandRank {
    fn from(value: [Card; 5]) -> Self {
        if is_royal_flush(&value) {
            return HandRank::RoyalFlush;
        } else if is_straight_flush(&value) {
            return HandRank::StraightFlush;
        } else if is_four_of_a_kind(&value) {
            return HandRank::FourOfAKind;
        } else if is_full_house(&value) {
            return HandRank::FullHouse;
        } else if is_flush(&value) {
            return HandRank::Flush;
        } else if is_straight(&value) {
            return HandRank::Straight;
        } else if is_three_of_a_kind(&value) {
            return HandRank::ThreeOfAKind;
        } else if is_two_pair(&value) {
            return HandRank::TwoPair;
        } else if is_pair(&value) {
            return HandRank::Pair;
        } else {
            return HandRank::HighCard;
        }

        fn is_royal_flush(cards: &[Card; 5]) -> bool {
            let mut ranks: Vec<CardRank> = cards.iter().map(|card| card.rank.clone()).collect();
            ranks.sort_by(|a, b| Ord::cmp(b, a));
            ranks == vec![CardRank::Ace, CardRank::King, CardRank::Queen, CardRank::Jack, CardRank::Ten]
        }

        fn is_straight_flush(cards: &[Card; 5]) -> bool {
            is_flush(cards) && is_straight(cards)
        }

        fn is_four_of_a_kind(cards: &[Card; 5]) -> bool {
            is_n_of_a_kind(cards, 4)
        }

        fn is_full_house(cards: &[Card; 5]) -> bool {
            let ranks: Vec<CardRank> = cards.iter().map(|c| c.rank.clone()).collect();
            let mut rank_counts = HashMap::<&CardRank, u8>::with_capacity(5);
            for r in ranks.iter() {
                let prev = rank_counts.get(r);
                match prev {
                    Some(prev) => {
                        let next = prev + 1;
                        rank_counts.insert(r, next)
                    },
                    _ => rank_counts.insert(r, 1)
                };
            }
            let counts: Vec<&u8> = rank_counts.values().collect();
            counts.contains(&&3) && counts.contains(&&2)
        }

        fn is_flush(cards: &[Card; 5]) -> bool {
            let suits: Vec<CardSuit> = cards.iter().map(|card| card.suit.clone()).collect();
            let suit_a = &suits[0];
            suits.iter().all(|suit| suit == suit_a)
        }

        fn is_straight(cards: &[Card; 5]) -> bool {
            let mut ranks: Vec<CardRank> = cards.iter().map(|card| card.rank.clone()).collect();
            ranks.sort();
            let has_ace = ranks.contains(&CardRank::Ace);
            let has_king = ranks.contains(&CardRank::King);
            let has_two = ranks.contains(&CardRank::Two);

            fn test_straight(mut ranks: Vec<CardRank>, low_ace: bool) -> bool {
                if low_ace {
                    ranks = ranks.shift_ace().unwrap();
                }
                for ranks in ranks.windows(2) {
                    let next_actual = match ranks[0].clone().next() {
                        Some(rank) => rank,
                        _ => {
                            if low_ace {
                                CardRank::Two
                            } else {
                                return false; // High Ace may not be succeeded in a Straight
                            }
                        }
                    };
                    if next_actual != ranks[1] {
                        return false;
                    }
                }
                true
            }

            match (has_ace, has_king, has_two) {
                (true, true, true) => false,
                (true, false, true) => test_straight(ranks, true),
                _ => test_straight(ranks, false),
            }
        }

        fn is_three_of_a_kind(cards: &[Card; 5]) -> bool {
            is_n_of_a_kind(cards, 3)
        }

        fn is_two_pair(cards: &[Card; 5]) -> bool {
            let ranks: Vec<CardRank> = cards.iter().map(|c| c.rank.clone()).collect();
            let mut rank_counts = HashMap::<&CardRank, u8>::with_capacity(5);
            let mut has_pairs = 0;
            for r in ranks.iter() {
                let prev = rank_counts.get(r);
                match prev {
                    Some(prev) => {
                        let next = prev + 1;
                        if next == 2 {
                            has_pairs += 1;
                            if has_pairs == 2 {
                                break;
                            }
                        }
                        rank_counts.insert(r, next)
                    },
                    _ => rank_counts.insert(r, 1)
                };
            }
            has_pairs == 2
        }

        fn is_pair(cards: &[Card; 5]) -> bool {
            is_n_of_a_kind(cards, 2)
        }

        fn is_n_of_a_kind(cards: &[Card; 5], n: u8) -> bool {
            assert!(n > 1 && n < 5);
            let ranks: Vec<CardRank> = cards.iter().map(|c| c.rank.clone()).collect();
            let mut rank_counts = HashMap::<&CardRank, u8>::with_capacity(5);
            let mut has_n = false;
            for r in ranks.iter() {
                let prev = rank_counts.get(r);
                match prev {
                    Some(prev) => {
                        let next = prev + 1;
                        if next == n {
                            has_n = true;
                            break;
                        }
                        rank_counts.insert(r, next)
                    },
                    _ => rank_counts.insert(r, 1)
                };
            }
            has_n
        }
    }
}

impl Ord for HandRank {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }
        let (mut self_rank, mut other_rank) = (0, 0);
        for (i, rank) in Self::ORDER.iter().enumerate() {
            if rank == self {
                self_rank = i;
            }
            if rank == other {
                other_rank = i;
            }
            if self_rank != 0 && other_rank != 0 {
                break;
            }
        }
        if self_rank < other_rank {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl PartialOrd for HandRank {
    // https://github.com/rust-lang/rust/issues/63104
    // https://github.com/rust-lang/rfcs/pull/1028
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
