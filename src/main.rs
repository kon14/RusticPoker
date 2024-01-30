use crate::types::card::Card;

mod types;
mod utils;

fn main() {
    let raw_card = "AS"; // Ace of Spades
    let card = Card::try_from(raw_card).unwrap();
    println!("{:?}", card);
}
