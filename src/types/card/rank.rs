use std::cmp::Ordering;
use crate::types::card::CardParseError;

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub(crate) enum CardRank {
    Ace,
    King,
    Queen,
    Jack,
    Ten,
    Nine,
    Eight,
    Seven,
    Six,
    Five,
    Four,
    Three,
    Two,
}

impl CardRank {
    const ORDER: [Self; 13] = [
        Self::Ace,
        Self::King,
        Self::Queen,
        Self::Jack,
        Self::Ten,
        Self::Nine,
        Self::Eight,
        Self::Seven,
        Self::Six,
        Self::Five,
        Self::Four,
        Self::Three,
        Self::Two,
    ];
}

impl<'a> TryFrom<&'a str> for CardRank {
    type Error = CardParseError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let rank_str: &str = &value.to_uppercase();
        match rank_str {
            "A" => Ok(Self::Ace),
            "K" => Ok(Self::King),
            "Q" => Ok(Self::Queen),
            "J" => Ok(Self::Jack),
            "10" => Ok(Self::Ten),
            "9" => Ok(Self::Nine),
            "8" => Ok(Self::Eight),
            "7" => Ok(Self::Seven),
            "6" => Ok(Self::Six),
            "5" => Ok(Self::Five),
            "4" => Ok(Self::Four),
            "3" => Ok(Self::Three),
            "2" => Ok(Self::Two),
            _ => Err(CardParseError::InvalidRank(value)),
        }
    }
}

impl Ord for CardRank {
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

impl PartialOrd<Self> for CardRank {
    // https://github.com/rust-lang/rust/issues/63104
    // https://github.com/rust-lang/rfcs/pull/1028
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
