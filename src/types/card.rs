use thiserror::Error;

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

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub(crate) enum CardSuit {
    Diamonds,
    Hearts,
    Clubs,
    Spades,
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub(crate) struct Card {
    rank: CardRank,
    suit: CardSuit,
}

#[derive(Error, Debug)]
pub(crate) enum CardParseError<'a> {
    #[error("Invalid card rank: `{0}`")]
    InvalidRank(&'a str),
    #[error("Invalid card suit: `{0}`")]
    InvalidSuit(&'a str),
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

impl<'a> TryFrom<&'a str> for CardSuit {
    type Error = CardParseError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let suit_str: &str = &value.to_uppercase();
        match suit_str {
            "D" => Ok(Self::Diamonds),
            "H" => Ok(Self::Hearts),
            "C" => Ok(Self::Clubs),
            "S" => Ok(Self::Spades),
            _ => Err(CardParseError::InvalidSuit(value)),
        }
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
