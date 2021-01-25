#![allow(unused_imports)]
#![allow(dead_code)]
use crate::gamestate::{
    ChipPile, ClientEvent, FromPlayer, GameAction, GameError, GameState, PlayerID,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::option::Option;
use time::Time;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameCoordinator {
    available_players: Vec<PlayerID>,
    current_games: Vec<GameState>,
    last_player_input: HashMap<PlayerID, Time>,
    player_money: HashMap<PlayerID, ChipPile>,
    events_to_send: HashMap<PlayerID, Vec<ClientEvent>>,
}

#[derive(Debug)]
pub enum CoordinatorError {
    GameError(GameError),
    PlayerNotFound,
}

impl From<GameError> for CoordinatorError {
    fn from(error: GameError) -> Self {
        CoordinatorError::GameError(error)
    }
}

impl GameCoordinator {
    pub fn new() -> Self {
        Self {
            available_players: Vec::new(),
            current_games: Vec::new(),
            last_player_input: HashMap::new(),
            player_money: HashMap::new(),
            events_to_send: HashMap::new(),
        }
    }

    pub fn on_new_user(&mut self) -> PlayerID {
        let id = PlayerID::new_v4();
        self.available_players.push(id);
        if self.available_players.len() == 4 {
            self.current_games
                .push(GameState::new(self.available_players.clone()));
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

    pub fn handle_action(
        &mut self,
        player_id: PlayerID,
        action: GameAction,
    ) -> Result<Vec<ClientEvent>, CoordinatorError> {
        let player_game = self
            .get_mut_current_games()
            .iter_mut()
            .filter(|game| game.get_player_list().contains(&&player_id))
            .next()
            .ok_or(CoordinatorError::PlayerNotFound)?;
        let client_event = player_game.action(action, player_id)?;
        let players = player_game.get_player_list().clone();
        drop(player_game);
        players
            .iter()
            .filter(|id| id != &&player_id)
            .for_each(|id| {
                self.events_to_send
                    .entry(*id)
                    .and_modify(|events| events.extend(client_event.clone()))
                    .or_insert(client_event.clone());
            });
        Ok(client_event)
    }

    pub fn get_other_events(&mut self) -> HashMap<PlayerID, Vec<ClientEvent>> {
        let client_event = self.events_to_send.clone();
        self.events_to_send.clear();
        client_event
    }

    pub fn get_available_players(&self) -> &Vec<PlayerID> {
        &self.available_players
    }

    pub fn get_current_games(&self) -> &Vec<GameState> {
        &self.current_games
    }

    pub fn get_mut_current_games(&mut self) -> &mut Vec<GameState> {
        &mut self.current_games
    }

    pub fn get_last_player_input(&self) -> &HashMap<PlayerID, Time> {
        &self.last_player_input
    }

    pub fn get_player_money(&self) -> &HashMap<PlayerID, ChipPile> {
        &self.player_money
    }

    pub fn get_events_to_send(&self) -> &HashMap<PlayerID, Vec<ClientEvent>> {
        &self.events_to_send
    }
}
