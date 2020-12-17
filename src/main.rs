use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem;
use uuid::Uuid;


type ChipPile = i32;
type Hand = Vec<i32>;
type PlayerID = Uuid;

#[derive(Debug, Deserialize, Serialize)]
struct GameState {
    current_player: Option<PlayerID>,
    player_list: Vec<PlayerID>,
    player_hand: HashMap<PlayerID, Hand>,
    player_money: HashMap<PlayerID, ChipPile>,
    player_bet: HashMap<PlayerID, ChipPile>,
    dealer_hand: Hand,
    deck: Vec<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
enum GameAction {
    Hit,
    Double,
    AddMoney(ChipPile),
    StartingBet(ChipPile),
}

impl GameState {
    fn new() -> Self {
        Self {
            current_player: None,
            player_list: Vec::new(),
            player_hand: HashMap::new(),
            player_money: HashMap::new(),
            player_bet: HashMap::new(),
            dealer_hand: Vec::new(),
            deck: vec![11, 10, 10, 10],
        }
    }

    fn new_user(&mut self) -> PlayerID {
        let id = Uuid::new_v4();
        self.player_list.push(id);
        self.player_hand.insert(id, Vec::new());
        self.player_money.insert(id, 0);

        id
    }

    fn is_current_player(&mut self, player: PlayerID) {
        self.current_player = Some(player); 
    }

    fn ace_conversion(&mut self, player: PlayerID, mut sum: i32) {
        let mut ace_index = self.player_hand            
        .get(&player)
        .unwrap()
        .iter()
        .position(|&x| x == 11);
    while sum > 21 && ace_index != None {
        let _ = mem::replace(&mut self.player_hand.get_mut(&player).unwrap()[ace_index.unwrap()], 1);
        sum -= 10;
        ace_index = self.player_hand            
        .get(&player)
        .unwrap()
        .iter()
        .position(|&x| x == 11);
        }
    }

    fn sum_hand(&mut self, player: PlayerID) -> i32 {
        let sum = self.player_hand            
            .get(&player)
            .unwrap()
            .iter()
            .sum();
        GameState::ace_conversion(self, player, sum);

        sum
    }

    fn action(&mut self, event: GameAction, player: PlayerID) {
        match event {
            GameAction::Hit => {
                if self.current_player == Some(player) {
                    self.player_hand.entry(player).or_insert(Vec::new()).push(self.deck[0]);
                    self.deck.remove(0);
                    if GameState::sum_hand(self, player) > 21 {
                        self.player_bet.remove(&player);
                    }
                }
            },
            GameAction::Double => {
                // Cannot Double with more than 2 cards
                if self.current_player == Some(player) {
                    let bet = self.player_bet.get(&player).unwrap();
                    if bet <= self.player_money.get(&player).unwrap() {
                        *self.player_money.entry(player).or_insert(0) -= bet;
                        *self.player_bet.entry(player).or_insert(0) *= 2;
                        GameState::action(self, GameAction::Hit, player);
                    }
                }
            },
            GameAction::AddMoney(value) => *self.player_money.entry(player).or_insert(0) += value,
            GameAction::StartingBet(bet) => {
                if bet <= *self.player_money.get(&player).unwrap() {
                    self.player_bet.insert(player, bet);
                    *self.player_money.entry(player).or_insert(0) -= bet;
                }
            },
        }                
    }
}

fn main() {
    // Testing
    let mut game: GameState = GameState::new();
    let player1 = GameState::new_user(&mut game);
    game.is_current_player(player1);
    game.action(GameAction::AddMoney(100), player1);
    println!("{:?}", game);
    game.action(GameAction::StartingBet(50), player1);
    game.action(GameAction::Hit, player1);
    game.action(GameAction::Hit, player1);
    game.action(GameAction::Double, player1);
    println!("{:?}", game);
}
