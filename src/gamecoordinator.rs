#![allow(unused_imports)]
#![allow(dead_code)]
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
    events_to_send: Vec<(PlayerID, ClientEvent)>,
}

impl GameCoordinator {
    pub fn new() -> Self {
        Self {
            available_players: Vec::new(),
            current_games: Vec::new(),
            last_player_input: HashMap::new(),
            player_money: HashMap::new(),
            events_to_send: Vec::new(),
        }
    }

    pub fn on_new_user(&mut self) -> PlayerID {
        let id = PlayerID::new_v4();
        self.available_players.push(id);
        if self.available_players.len() == 4 {
            self.current_games.push(GameState::new(&self.available_players));
            self.available_players.clear();
        }

        id
    }

    pub fn on_dropped_user(&mut self, player_id: PlayerID) {  
        if self.available_players.contains(&player_id) { 
            self.available_players.retain(|&x| x != player_id);
        } else {
            for game in self.get_mut_current_games() {
                game.remove_user(player_id);
            }
        }
    }

    pub fn handle_action(&mut self, player_id: PlayerID, action: GameAction) -> Vec<ClientEvent> {
        for game in self.get_mut_current_games() {
            if game.get_player_list().contains(&player_id) {
                // game.action(action, player_id).ok();
            }
        }
        todo!();
    }

    pub fn get_other_events(&mut self) -> Vec<(PlayerID, ClientEvent)> {
        self.events_to_send.clear();
        todo!();
    }

    pub fn get_mut_current_games(&mut self) -> &mut Vec<GameState> {
        &mut self.current_games
    }
}
