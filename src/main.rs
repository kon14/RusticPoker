use crate::types::{
    card::Card,
    hand::{Hand, HandRank},
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
    // HandRank
    let cards = [
      Card::try_from("10C").unwrap(),
      Card::try_from("10D").unwrap(),
      Card::try_from("10C").unwrap(),
      Card::try_from("10D").unwrap(),
      Card::try_from("AH").unwrap(),
    ];
    let hand_rank: HandRank = cards.into();
    println!("{:?}", hand_rank);
    let cards = [
        Card::try_from("AC").unwrap(),
        Card::try_from("KS").unwrap(),
        Card::try_from("QD").unwrap(),
        Card::try_from("JC").unwrap(),
        Card::try_from("10H").unwrap(),
    ];
    let hand_rank: HandRank = cards.into();
    println!("{:?}", hand_rank);
    let cards = [
        Card::try_from("AS").unwrap(),
        Card::try_from("2C").unwrap(),
        Card::try_from("3D").unwrap(),
        Card::try_from("4D").unwrap(),
        Card::try_from("5C").unwrap(),
    ];
    let hand_rank: HandRank = cards.into();
    println!("{:?}", hand_rank);
    let cards = [
        Card::try_from("AS").unwrap(),
        Card::try_from("2C").unwrap(),
        Card::try_from("KD").unwrap(),
        Card::try_from("QD").unwrap(),
        Card::try_from("JC").unwrap(),
    ];
    let hand_rank: HandRank = cards.into();
    println!("{:?}", hand_rank);
    // Hand
    let h1: Hand = "AS KS QS JS 10S".try_into().unwrap();
    let h2: Hand = "AS 2S 3S 4S 5S".try_into().unwrap();
    // let h4: Hand = "AS KS QS JS".try_into().unwrap();
    // let h5: Hand = "AS KS QS JS 10S 9S".try_into().unwrap();
    println!("{:?}", h1);
    println!("{:?}", h2);
}
