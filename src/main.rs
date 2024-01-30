use crate::types::card::Card;

mod types;
mod utils;

fn main() {
    let raw_card = "AS"; // Ace of Spades
    let card_as = Card::try_from(raw_card).unwrap();
    let raw_card = "AH"; // Ace of Hearts
    let card_ah = Card::try_from(raw_card).unwrap();
    let raw_card = "2C"; // Two of Clubs
    let card_2c = Card::try_from(raw_card).unwrap();
    if card_as == card_ah {
        println!("{:?} == {:?}", card_as, card_ah);
    } else if card_as > card_ah {
        println!("{:?} > {:?}", card_as, card_ah);
    } else {
        println!("{:?} < {:?}", card_as, card_ah);
    }
    if card_as == card_2c {
        println!("{:?} == {:?}", card_as, card_2c);
    } else if card_as > card_2c {
        println!("{:?} > {:?}", card_as, card_2c);
    } else {
        println!("{:?} < {:?}", card_as, card_2c);
    }
}
