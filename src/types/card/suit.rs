use crate::types::card::CardParseError;

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub(crate) enum CardSuit {
    Diamonds,
    Hearts,
    Clubs,
    Spades,
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
