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
    pub fn new() {
        todo!();
    }

    pub fn on_new_user() {
        todo!();
    }

    pub fn on_dropped_user() {
        todo!();
    }

    pub fn handle_action(player_id: PlayerID, action: GameAction) {
        todo!();
    }
}
