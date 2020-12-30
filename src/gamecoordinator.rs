use crate::gamestate::{
    ChipPile, ClientEvent, FromPlayer, GameAction, GameError, GameState, PlayerID,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::Time;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameCoordinator {
    available_players: Vec<PlayerID>,
    current_games: Vec<GameState>,
    last_player_input: HashMap<PlayerID, Time>,
    player_money: HashMap<PlayerID, ChipPile>,
}

impl GameCoordinator {
    pub fn new() -> Self {
        todo!();
    }

    pub fn on_new_user(&mut self) -> PlayerID {
        todo!();
    }

    pub fn on_dropped_user(&mut self, player_id: PlayerID) {
        todo!();
    }

    pub fn handle_action(&mut self, player_id: PlayerID, action: GameAction) -> Vec<ClientEvent> {
        todo!();
    }
}
