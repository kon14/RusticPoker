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
        Some(Ord::cmp(&self, &other))
    }
}

impl Iterator for CardRank {
    type Item = CardRank;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CardRank::Two => Some(CardRank::Three),
            CardRank::Three => Some(CardRank::Four),
            CardRank::Four => Some(CardRank::Five),
            CardRank::Five => Some(CardRank::Six),
            CardRank::Six => Some(CardRank::Seven),
            CardRank::Seven => Some(CardRank::Eight),
            CardRank::Eight => Some(CardRank::Nine),
            CardRank::Nine => Some(CardRank::Ten),
            CardRank::Ten => Some(CardRank::Jack),
            CardRank::Jack => Some(CardRank::Queen),
            CardRank::Queen => Some(CardRank::King),
            CardRank::King => Some(CardRank::Ace),
            CardRank::Ace => None,
        }
    }
}

impl<'a> Iterator for &'a CardRank {
    type Item = &'a CardRank;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CardRank::Two => Some(&CardRank::Three),
            CardRank::Three => Some(&CardRank::Four),
            CardRank::Four => Some(&CardRank::Five),
            CardRank::Five => Some(&CardRank::Six),
            CardRank::Six => Some(&CardRank::Seven),
            CardRank::Seven => Some(&CardRank::Eight),
            CardRank::Eight => Some(&CardRank::Nine),
            CardRank::Nine => Some(&CardRank::Ten),
            CardRank::Ten => Some(&CardRank::Jack),
            CardRank::Jack => Some(&CardRank::Queen),
            CardRank::Queen => Some(&CardRank::King),
            CardRank::King => Some(&CardRank::Ace),
            CardRank::Ace => None,
        }
    }
}

pub(crate) trait ShiftAce {
    fn shift_ace(&self) -> Option<Self> where Self: Sized;
}

impl ShiftAce for Vec<CardRank> {
    /// Provided source contains an ace, returns a low-ace, asc-sorted vector of card ranks.
    fn shift_ace(&self) -> Option<Self> {
        if !self.contains(&CardRank::Ace) {
            return None;
        }
        let mut ranks = self.clone();
        ranks.sort();
        let ace = ranks.pop().unwrap();
        ranks.insert(0, ace);
        Some(ranks)
    }
}

pub(crate) trait GroupByRank {
    fn group_by_rank(&self) -> Vec<Self> where Self: Sized;
}

impl GroupByRank for Vec<CardRank> {
    fn group_by_rank(&self) -> Vec<Self> {
        let mut result = Vec::new();
        let mut current_group = Vec::new();
        let mut ranks = self.clone();
        ranks.sort();
        for (i, rank) in ranks.iter().enumerate() {
            if i > 0 && *rank != ranks[i - 1] {
                result.push(std::mem::replace(&mut current_group, Vec::new()));
            }
            current_group.push(rank.clone());
        }
        if !current_group.is_empty() {
            result.push(current_group);
        }
        result
    }
}
