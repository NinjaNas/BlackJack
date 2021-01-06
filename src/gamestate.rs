#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::option::Option;
use uuid::Uuid;

pub type ChipPile = i32;
pub type Hand = Vec<i32>;
pub type PlayerID = Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameState {
    current_player: Option<PlayerID>,
    player_list: Vec<PlayerID>,
    player_hand: HashMap<PlayerID, Hand>,
    player_money: HashMap<PlayerID, ChipPile>,
    player_bet: HashMap<PlayerID, ChipPile>,
    dealer_hand: Hand,
    deck: Vec<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum GameAction {
    Hit,
    Stand,
    Double,
    AddMoney(ChipPile),
    StartingBet(ChipPile),
}

#[derive(Debug)]
pub enum GameError {
    MissingPlayerID,
    InvaildAction,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum FromPlayer {
    Dealer,
    Player(PlayerID),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum ClientEvent {
    RoundOver,
    CardRevealed(FromPlayer, i32),
    Betting(PlayerID, ChipPile),
}

impl GameState {
    pub fn new(users: &Vec<PlayerID>) -> Self {
        Self {
            current_player: None,
            player_list: users.to_vec(),
            player_hand: HashMap::new(),
            player_money: HashMap::new(),
            player_bet: HashMap::new(),
            dealer_hand: Vec::new(),
            deck: vec![2, 8, 11, 10, 10, 10, 10, 7],
        }
    }

    pub fn create_users_hand(&mut self) {
        for id in &self.player_list {
            self.player_hand.insert(*id, Vec::new());
            self.player_money.insert(*id, 0);
        }
    }

    pub fn remove_user(&mut self, player: PlayerID) -> PlayerID {
        self.player_list.retain(|&x| x != player);
        self.player_hand.retain(|x, _| *x != player);
        self.player_money.retain(|x, _| *x != player);

        player
    }

    pub fn start_game(&mut self) {
        if self.current_player == None {
            self.current_player = Some(self.player_list[0]);
            // for player in self.player_list {
            //     self.action(GameAction::Hit, player).ok();
            // }
        }
    }

    pub fn next_current_player(&mut self, player: PlayerID) -> Result<(), GameError> {
        let mut iter = self.player_list.iter();
        let _ = iter.by_ref().find(|&&id| player == id);
        Ok(self.current_player = Some(*iter.next().ok_or(GameError::MissingPlayerID)?))
    }

    pub fn get_current_player(&self) -> Option<PlayerID> {
        self.current_player
    }

    pub fn get_player_list(&self) -> &Vec<PlayerID> {
        &self.player_list
    }

    pub fn get_dealer_hand(& self) -> &Hand {
        &self.dealer_hand
    }

    pub fn get_player_hand(&self, player: PlayerID) -> Result<&Hand, GameError> {
        self.player_hand
            .get(&player)
            .ok_or(GameError::MissingPlayerID)
    }

    pub fn get_mut_player_hand(&mut self, player: PlayerID) -> Result<&mut Hand, GameError> {
        self.player_hand
            .get_mut(&player)
            .ok_or(GameError::MissingPlayerID)
    }

    pub fn get_player_money(&self, player: PlayerID) -> Result<i32, GameError> {
        self.player_money
            .get(&player)
            .ok_or(GameError::MissingPlayerID)
            .map(|money| *money)
    }

    pub fn get_mut_player_money(&mut self, player: PlayerID) -> Result<&mut i32, GameError> {
        self.player_money
            .get_mut(&player)
            .ok_or(GameError::MissingPlayerID)
    }

    pub fn set_player_money(&mut self, player: PlayerID, value: i32) -> Result<(), GameError> {
        *self
            .player_money
            .get_mut(&player)
            .ok_or(GameError::MissingPlayerID)? = value;
        Ok(())
    }

    pub fn get_player_bet(&self, player: PlayerID) -> Result<i32, GameError> {
        self.player_bet
            .get(&player)
            .ok_or(GameError::MissingPlayerID)
            .map(|bet| *bet)
    }

    pub fn get_mut_player_bet(&mut self, player: PlayerID) -> Result<&mut i32, GameError> {
        self.player_bet
            .get_mut(&player)
            .ok_or(GameError::MissingPlayerID)
    }

    pub fn set_player_bet(&mut self, player: PlayerID, value: i32) -> Result<(), GameError> {
        *self
            .player_bet
            .get_mut(&player)
            .ok_or(GameError::MissingPlayerID)? = value;
        Ok(())
    }

    pub fn is_round_over(&mut self, event: Vec<ClientEvent>) -> bool {
        event.iter().any(|x| match x {
            ClientEvent::RoundOver => true,
            _ => false,
        })
    }

    fn ace_conversion(&mut self, player: PlayerID, mut sum: i32) -> Result<i32, GameError> {
        let ace_count = self
            .get_player_hand(player)?
            .iter()
            .filter(|&n| *n == 11)
            .count();
        while sum > 21 && ace_count > 0 {
            sum -= 10;
        }
        Ok(sum)
    }

    pub fn sum_hand(&mut self, player: PlayerID) -> Result<i32, GameError> {
        let mut sum = self.get_player_hand(player)?.iter().sum();
        if sum > 21 {
            sum = self.ace_conversion(player, sum)?;
        }

        Ok(sum)
    }

    pub fn sum_dealer(&mut self) -> i32 {
        self.dealer_hand.iter().sum()
    }

    pub fn dealer_draw(&mut self) {
        let new_card = self.deck.remove(0);
        self.dealer_hand.push(new_card);
    }

    pub fn dealer_draw_final(&mut self) {
        let new_card = self.deck.remove(0);
        while self.sum_dealer() < 17 {
            self.dealer_hand.push(new_card);
        }
    }

    pub fn compare_hands(&mut self) -> Result<(), GameError> {
        // iter_mut, if bet value is 0 continue for loop
        let mut clone_bet = self.player_bet.clone();
        clone_bet.retain(|_, x| *x != 0);

        let dealer_sum = self.sum_dealer();

        for key in clone_bet.keys() {
            if dealer_sum <= 21 && self.sum_hand(*key)? < dealer_sum {
                *self.get_mut_player_bet(*key)? *= 0;
            } else {
                *self.get_mut_player_bet(*key)? *= 2;
            }
        }
        Ok(())
    }

    pub fn return_bet(&mut self) -> Result<(), GameError> {
        let clone_bet = self.player_bet.clone();

        for (key, val) in clone_bet.iter() {
            *self.get_mut_player_money(*key)? += val;
        }
        Ok(self.player_bet.clear())
    }

    pub fn action(
        &mut self,
        event: GameAction,
        player: PlayerID,
    ) -> Result<Vec<ClientEvent>, GameError> {
        match event {
            GameAction::Hit if self.current_player == Some(player) => {
                let new_card = self.deck.remove(0);
                self.get_mut_player_hand(player)?.push(new_card);
                if self.sum_hand(player)? > 21 {
                    *self.get_mut_player_bet(player)? *= 0;
                    self.action(GameAction::Stand, player).ok();
                }
                Ok(vec![ClientEvent::CardRevealed(
                    FromPlayer::Player(player),
                    new_card,
                )])
            }
            GameAction::Stand if self.current_player == Some(player) => {
                let next_player = self.next_current_player(player).ok();
                if next_player == None {
                    self.dealer_draw_final();
                    self.compare_hands().ok();
                    self.return_bet().ok();
                }
                Ok(vec![ClientEvent::RoundOver])
            }
            GameAction::Double
                if self.current_player == Some(player)
                    && self.get_player_hand(player)?.len() == 2 =>
            {
                // First two cards equal to 9, 10, or 11
                let bet = self.get_player_bet(player)?;
                if bet <= self.get_player_money(player)? {
                    *self.get_mut_player_money(player)? -= bet;
                    *self.get_mut_player_bet(player)? *= 2;
                    self.action(GameAction::Hit, player).ok();
                }
                Ok(vec![ClientEvent::RoundOver])
            }
            GameAction::AddMoney(value) if value > 0 => {
                *self.get_mut_player_money(player)? += value;
                Ok(vec![ClientEvent::Betting(player, value)])
            }
            GameAction::StartingBet(bet) if bet > 0 && bet <= self.get_player_money(player)? => {
                self.player_bet.insert(player, bet);
                *self.get_mut_player_money(player)? -= bet;
                if self.player_list.len() == self.player_bet.len() {
                    self.start_game();
                }
                Ok(vec![ClientEvent::Betting(player, bet)])
            }
            _ => Err(GameError::InvaildAction),
        }
    }
}
