use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct Card {
    card_suit: CardSuit,
    value: Value,
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&*format!("{}{:?}", self.card_suit.to_symbol(), self.value))
    }
}

#[test]
fn debug_output_for_card() {
    let card = Card::new(CardSuit::Spades, Value::Two);
    assert_eq!(format!("{:?}", card), "♠Two".to_string());
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CardSuit {
    Spades,
    Hearts,
    Diamond,
    Clubs,
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
    Ace,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Value {
    pub fn value(&self) -> i8 {
        match &self {
            Value::Two => 2,
            Value::Three => 3,
            Value::Four => 4,
            Value::Five => 5,
            Value::Six => 6,
            Value::Seven => 7,
            Value::Eight => 8,
            Value::Nine => 9,
            Value::Ten => 10,
            Value::Jack => 10,
            Value::Queen => 10,
            Value::King => 10,
            Value::Ace => 11,
        }
    }
}

impl CardSuit {
    pub fn to_symbol(&self) -> String {
        match self {
            CardSuit::Spades => "♠".to_string(),
            CardSuit::Hearts => "❤️".to_string(),
            CardSuit::Diamond => "♦".to_string(),
            CardSuit::Clubs => "♣".to_string(),
        }
    }
}

impl Card {
    pub fn is_face(&self) -> bool {
        match self.value {
            Value::Jack | Value::Queen | Value::King | Value::Ace => true,
            _ => false,
        }
    }

    pub fn is_ace(&self) -> bool {
        match self.value {
            Value::Ace => true,
            _ => false,
        }
    }

    pub fn new(suit: CardSuit, value: Value) -> Self {
        Self {
            card_suit: suit,
            value,
        }
    }
}
