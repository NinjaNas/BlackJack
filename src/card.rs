use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    card_suit: CardSuit,
    value: Value
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CardSuit {
    Spades,
    Hearts,
    Diamond,
    Clubs
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deck {
    cards: Vec<Card>
}

impl Value {
    pub fn value(self) -> i8 {
        match self {
            Two => 2,
            Three => 3,
            Four => 4,
            Five => 5,
            Six => 6,
            Seven => 7,
            Eight => 8,
            Nine => 9,
            Ten => 10,
            Jack => 10,
            Queen => 10,
            King => 10,
            Ace => 11
        }
    }
}

impl CardSuit {
    pub fn to_symbol(self) -> String {
        match self {
            Spades => 's',
            Hearts => 'h',
            Diamond => 'd',
            Clubs => 'c'
        }
    }
}

impl Card {
    pub fn is_face(self) -> bool {
        match self {
            Jack, Queen, King, Ace => true
            _ => false
        }
    }

    pub fn is_ace(self) -> bool {
        match self {
            Ace => true
            _ => false
        }
    }

}