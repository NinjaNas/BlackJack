use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::option::Option;
use uuid::Uuid;

type ChipPile = i32;
type Hand = Vec<i32>;
type PlayerID = Uuid;

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

#[derive(Debug, Deserialize, Serialize)]
pub enum FromPlayer {
    Dealer,
    Player(PlayerID),
}

#[derive(Debug, Deserialize, Serialize)]
enum ClientEvent {
    NewRound,
    CardRevealed(FromPlayer, i32),
    Betting(PlayerID, ChipPile),
}

impl GameState {
    pub fn new() -> Self {
        Self {
            current_player: None,
            player_list: Vec::new(),
            player_hand: HashMap::new(),
            player_money: HashMap::new(),
            player_bet: HashMap::new(),
            dealer_hand: Vec::new(),
            deck: vec![11, 10, 10, 10, 10, 7],
        }
    }

    pub fn add_user(&mut self) -> PlayerID {
        let id = Uuid::new_v4();
        self.player_list.push(id);
        self.player_hand.insert(id, Vec::new());
        self.player_money.insert(id, 0);

        id
    }

    pub fn assign_current_player(&mut self) {
        if self.current_player == None {
            self.current_player = Some(self.player_list[0]); 
        }
    }

    pub fn next_current_player(&mut self, player: PlayerID) -> Result<(), GameError> {
        let mut iter = self.player_list.iter();
        let _ = iter.by_ref().find(|&&id| player == id);
        self.current_player = Some(*iter.next().ok_or(GameError::MissingPlayerID)?);
        Ok(())
    }

    pub fn get_player_hand(&mut self, player: PlayerID) -> Result<&Hand, GameError> {
        self.player_hand.get(&player).ok_or(GameError::MissingPlayerID)
    }

    pub fn get_mut_player_hand(&mut self, player: PlayerID) -> Result<&mut Hand, GameError> {
        self.player_hand.get_mut(&player).ok_or(GameError::MissingPlayerID)
    }

    pub fn get_player_money(&mut self, player: PlayerID) -> Result<i32, GameError> {
        self.player_money.get(&player).ok_or(GameError::MissingPlayerID).map(|money| *money)
    }

    pub fn get_mut_player_money(&mut self, player: PlayerID) -> Result<&mut i32, GameError> {
        self.player_money.get_mut(&player).ok_or(GameError::MissingPlayerID)
    }

    pub fn set_player_money(&mut self, player: PlayerID, value: i32) -> Result<(), GameError> {
        *self.player_money.get_mut(&player).ok_or(GameError::MissingPlayerID)? = value;
        Ok(())
    }

    pub fn get_player_bet(&mut self, player: PlayerID) -> Result<i32, GameError> {
        self.player_bet.get(&player).ok_or(GameError::MissingPlayerID).map(|bet| *bet)
    }

    pub fn get_mut_player_bet(&mut self, player: PlayerID) -> Result<&mut i32, GameError> {
        self.player_bet.get_mut(&player).ok_or(GameError::MissingPlayerID)
    }

    pub fn set_player_bet(&mut self, player: PlayerID, value: i32) -> Result<(), GameError> {
        *self.player_bet.get_mut(&player).ok_or(GameError::MissingPlayerID)? = value;
        Ok(())
    }

    pub fn ace_conversion(&mut self, player: PlayerID, mut sum: i32) -> Result<i32, GameError> {
        let ace_count = self.get_player_hand(player)?.iter().filter(|&n| *n == 11).count();
        while sum > 21 && ace_count > 0 {
            sum -= 10;
        }
        Ok(sum)
    }
    
    pub fn sum_hand(&mut self, player: PlayerID) -> Result<i32, GameError> {
        let mut sum = self.player_hand            
            .get(&player)
            .unwrap()
            .iter()
            .sum();
        if sum > 21 {
            sum = self.ace_conversion(player, sum)?;
        }

        Ok(sum)
    }
    fn action(&mut self, event: GameAction, player: PlayerID) -> Result<Vec<ClientEvent>, GameError> {
        match event {
            GameAction::Hit if self.current_player == Some(player) => {
                let new_card = self.deck.remove(0);
                self.get_mut_player_hand(player)?.push(new_card);
                if self.sum_hand(player)? > 21 {
                    *self.get_mut_player_bet(player)? *= 0;
                }
            Ok(vec![ClientEvent::CardRevealed(FromPlayer::Player(player), new_card)])
            },
            GameAction::Stand if self.current_player == Some(player) => {
                self.next_current_player(player).ok();
                Ok(vec![ClientEvent::NewRound])
            },
            GameAction::Double if self.current_player == Some(player) && self.get_player_hand(player)?.len() == 2 => {
                // First two cards equal to 9, 10, or 11
                let bet = self.get_player_bet(player)?;
                if bet <= self.get_player_money(player)? {
                    *self.get_mut_player_money(player)? -= bet;
                    *self.get_mut_player_bet(player)? *= 2;
                    let _ = GameState::action(self, GameAction::Hit, player);
                }
                Ok(vec![ClientEvent::NewRound])
            },
            GameAction::AddMoney(value) if value > 0 => {
                *self.get_mut_player_money(player)? += value;
                Ok(vec![ClientEvent::Betting(player, value)])
            },
            GameAction::StartingBet(bet) if bet > 0 && bet <= self.get_player_money(player)? => {
                self.player_bet.insert(player, bet);
                *self.get_mut_player_money(player)? -= bet;
                if self.player_list.len() == self.player_bet.len() {
                    self.assign_current_player();
                }
                Ok(vec![ClientEvent::Betting(player, bet)])
            },
            _ =>Err(GameError::InvaildAction)
        }                
    }
}

fn main() {
    // Testing
    let mut game: GameState = GameState::new();
    let player1 = game.add_user();
    let player2 = game.add_user();
    
    game.action(GameAction::AddMoney(100), player1).ok();
    game.action(GameAction::StartingBet(50), player1).ok();
    game.assign_current_player();
    game.action(GameAction::Hit, player1).ok();
    println!("{:?}", game);
    game.action(GameAction::Hit, player1).ok();
    println!("{:?}", game);
    game.action(GameAction::Hit, player1).ok();
    println!("{:?}", game);
    game.action(GameAction::Stand, player1).ok();
    game.action(GameAction::Stand, player2).ok();
    println!("{:?}", game);

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_add_money() -> Result<(), GameError> {
        let mut game: GameState = GameState::new();
        let player1 = game.add_user();
        game.action(GameAction::AddMoney(-100), player1).ok();
        assert_eq!(0, game.get_player_money(player1)?);
        Ok(())
    }

    #[test]
    fn over_bet_money() -> Result<(), GameError> {
        let mut game: GameState = GameState::new();
        let player1 = game.add_user();
        game.action(GameAction::AddMoney(100), player1).ok();
        game.action(GameAction::StartingBet(200), player1).ok();
        assert_eq!(100, game.get_player_money(player1)?);
        Ok(())
    }

    #[test]
    fn is_current_player_after_all_bet() -> Result<(), GameError> {
        let mut game: GameState = GameState::new();
        let player1 = game.add_user();
        let player2 = game.add_user();
        game.action(GameAction::AddMoney(100), player1).ok();
        game.action(GameAction::StartingBet(100), player1).ok();
        assert_eq!(None, game.current_player);
        game.action(GameAction::AddMoney(100), player2).ok();
        game.action(GameAction::StartingBet(100), player2).ok();
        assert_eq!(Some(player1), game.current_player);
        Ok(())
    }

    #[test]
    fn hit() -> Result<(), GameError> {
        let mut game: GameState = GameState::new();
        let player1 = game.add_user();
        game.action(GameAction::AddMoney(100), player1).ok();
        game.action(GameAction::StartingBet(100), player1).ok();
        game.action(GameAction::Hit, player1).ok();
        game.action(GameAction::Hit, player1).ok();
        assert_eq!(vec![11, 10], *game.get_player_hand(player1)?);
        Ok(())
    }

    #[test]
    fn stand() -> Result<(), GameError> {
        let mut game: GameState = GameState::new();
        let player1 = game.add_user();
        let player2 = game.add_user();
        game.action(GameAction::AddMoney(100), player1).ok();
        game.action(GameAction::StartingBet(100), player1).ok();
        game.action(GameAction::AddMoney(100), player2).ok();
        game.action(GameAction::StartingBet(100), player2).ok();
        game.action(GameAction::Hit, player1).ok();
        game.action(GameAction::Hit, player1).ok();
        game.action(GameAction::Stand, player1).ok();
        assert_eq!(Some(player2), game.current_player);
        Ok(())
    }

    // Needs work
    #[test]
    fn game() -> Result<(), GameError> {
        let mut game: GameState = GameState::new();
        let player1 = game.add_user();
        let player2 = game.add_user();
        game.action(GameAction::AddMoney(100), player1).ok();
        game.action(GameAction::StartingBet(100), player1).ok();
        game.action(GameAction::AddMoney(200), player2).ok();
        game.action(GameAction::StartingBet(50), player2).ok();
        game.action(GameAction::Hit, player1).ok();
        game.action(GameAction::Hit, player1).ok();
        game.action(GameAction::Stand, player1).ok();
        game.action(GameAction::Hit, player2).ok();
        game.action(GameAction::Hit, player2).ok();
        game.action(GameAction::Stand, player2).ok();
        // Check dealer hand after everyone stands 
        assert_eq!(Some(player2), game.current_player);
        Ok(())
    }
}