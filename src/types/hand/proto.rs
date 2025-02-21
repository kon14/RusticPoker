use crate::service::proto;
use super::HandRank;

impl From<HandRank> for proto::game_state::poker_phase::poker_phase_showdown::showdown_results::PokerHandRank {
    fn from(rank: HandRank) -> Self {
        match rank {
            HandRank::RoyalFlush => Self::RoyalFlush,
            HandRank::StraightFlush => Self::StraightFlush,
            HandRank::FourOfAKind => Self::FourOfAKind,
            HandRank::FullHouse => Self::FullHouse,
            HandRank::Flush => Self::Flush,
            HandRank::Straight => Self::Straight,
            HandRank::ThreeOfAKind => Self::ThreeOfAKind,
            HandRank::TwoPair => Self::TwoPair,
            HandRank::Pair => Self::Pair,
            HandRank::HighCard => Self::HighCard,
        }
    }
}
