use crate::types::{
    card::Card,
    hand::HandRank,
};

mod types;
mod utils;

fn main() {
    // Card
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
    // HandRank
    if HandRank::FullHouse == HandRank::FullHouse {
        println!("{:?} == {:?}", HandRank::FullHouse, HandRank::FullHouse);
    } else if HandRank::FullHouse > HandRank::FullHouse {
        println!("{:?} > {:?}", HandRank::FullHouse, HandRank::FullHouse);
    } else {
        println!("{:?} < {:?}", HandRank::FullHouse, HandRank::FullHouse);
    }
    if HandRank::FullHouse == HandRank::RoyalFlush {
        println!("{:?} == {:?}", HandRank::FullHouse, HandRank::RoyalFlush);
    } else if HandRank::FullHouse > HandRank::RoyalFlush {
        println!("{:?} > {:?}", HandRank::FullHouse, HandRank::RoyalFlush);
    } else {
        println!("{:?} < {:?}", HandRank::FullHouse, HandRank::RoyalFlush);
    }
}
