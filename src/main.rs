#![allow(unused_imports)]
#![allow(dead_code)]
use gamecoordinator::GameCoordinator;
use gamestate::{GameAction, GameState, PlayerID};
use lib::gamecoordinator;
use lib::gamestate;

fn main() {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use gamestate::{ClientEvent, FromPlayer, GameAction, GameError, GameState, PlayerID};
    use lib::card::{Card, CardSuit, Value};

    #[test]
    fn gamecoordinator() -> Result<(), GameError> {
        // Test GameCoordinator
        let mut coordinator = GameCoordinator::new();

        // After four players in waiting room, start new game
        assert_eq!(0, coordinator.get_mut_current_games().len());
        let player1 = coordinator.on_new_user();
        coordinator.on_new_user();
        coordinator.on_new_user();
        coordinator.on_new_user();
        assert_eq!(1, coordinator.get_mut_current_games().len());

        // Test GameState
        coordinator.get_mut_current_games()[0].create_users_hand();
        coordinator.get_mut_current_games()[0]
            .get_mut_deck()
            .append(&mut vec![
                Card::new(CardSuit::Hearts, Value::Ace),
                Card::new(CardSuit::Hearts, Value::Nine),
            ]);

        assert_eq!(
            4,
            coordinator.get_mut_current_games()[0]
                .get_player_list()
                .len()
        );
        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Ace),
                Card::new(CardSuit::Hearts, Value::Nine)
            ],
            *coordinator.get_mut_current_games()[0].get_deck()
        );

        // Player 2 is in waiting room
        let player2 = coordinator.on_new_user();

        assert_eq!(
            true,
            coordinator.get_mut_current_games()[0]
                .get_player_list()
                .contains(&player1)
        );
        assert_eq!(true, coordinator.get_available_players().contains(&player2));

        // Test on_dropped_user remove players
        coordinator.on_dropped_user(player1);
        coordinator.on_dropped_user(player2);

        assert_eq!(
            false,
            coordinator.get_mut_current_games()[0]
                .get_player_list()
                .contains(&player1)
        );
        assert_eq!(
            false,
            coordinator.get_available_players().contains(&player2)
        );

        Ok(())
    }

    #[test]
    fn remove_player_game() -> Result<(), GameError> {
        // Test remove users
        let player1 = PlayerID::new_v4();
        let player2 = PlayerID::new_v4();
        let mut game: GameState = GameState::new(vec![player1, player2]);
        game.create_users_hand();
        assert_eq!(true, game.get_player_list().contains(&player1));
        game.remove_user(player1);
        assert_eq!(false, game.get_player_list().contains(&player1));
        assert_eq!(true, game.get_player_list().contains(&player2));
        Ok(())
    }

    #[test]
    fn negative_add_money() -> Result<(), GameError> {
        // Cannot use AddMoney for a negative value
        let player1 = PlayerID::new_v4();
        let mut game: GameState = GameState::new(vec![player1]);
        game.create_users_hand();
        game.action(GameAction::AddMoney(-100.0), player1).ok();
        assert_eq!(0.0, game.get_player_money(player1)?);
        Ok(())
    }

    #[test]
    fn over_bet_money() -> Result<(), GameError> {
        // Cannot over bet money that you don't have
        let player1 = PlayerID::new_v4();
        let mut game: GameState = GameState::new(vec![player1]);
        game.create_users_hand();
        game.action(GameAction::AddMoney(100.0), player1).ok();
        game.action(GameAction::StartingBet(200.0), player1).ok();
        assert_eq!(100.0, game.get_player_money(player1)?);
        Ok(())
    }

    #[test]
    fn is_current_player_after_all_bet() -> Result<(), GameError> {
        // Makes sure the first player in player_list is the current_player after betting
        let player1 = PlayerID::new_v4();
        let player2 = PlayerID::new_v4();
        let mut game: GameState = GameState::new(vec![player1, player2]);
        game.create_users_hand();
        game.get_mut_deck().append(&mut vec![
            Card::new(CardSuit::Hearts, Value::Ace),
            Card::new(CardSuit::Hearts, Value::Ten),
            Card::new(CardSuit::Hearts, Value::Two),
            Card::new(CardSuit::Hearts, Value::Jack),
            Card::new(CardSuit::Hearts, Value::King),
            Card::new(CardSuit::Hearts, Value::Eight),
            Card::new(CardSuit::Hearts, Value::Queen),
        ]);
        game.action(GameAction::AddMoney(100.0), player1).ok();
        game.action(GameAction::StartingBet(100.0), player1).ok();
        assert_eq!(None, game.get_current_player());
        game.action(GameAction::AddMoney(100.0), player2).ok();
        game.action(GameAction::StartingBet(100.0), player2).ok();
        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Ace),
                Card::new(CardSuit::Hearts, Value::Jack)
            ],
            *game.get_player_hand(player1)?
        );
        assert_eq!(Some(player2), game.get_current_player());
        game.action(GameAction::Stand, player2).ok();
        Ok(())
    }

    #[test]
    fn natural_blackjack() -> Result<(), GameError> {
        // Test natural_blackjack action with one player
        let player1 = PlayerID::new_v4();
        let mut game: GameState = GameState::new(vec![player1]);
        game.create_users_hand();
        game.get_mut_deck().append(&mut vec![
            Card::new(CardSuit::Hearts, Value::Ace),
            Card::new(CardSuit::Hearts, Value::Two),
            Card::new(CardSuit::Hearts, Value::Ten),
            Card::new(CardSuit::Hearts, Value::King),
            Card::new(CardSuit::Hearts, Value::Six),
        ]);
        game.action(GameAction::AddMoney(100.0), player1).ok();
        game.action(GameAction::StartingBet(100.0), player1).ok();

        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Ace),
                Card::new(CardSuit::Hearts, Value::Ten)
            ],
            *game.get_player_hand(player1)?
        );
        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Two),
                Card::new(CardSuit::Hearts, Value::King),
                Card::new(CardSuit::Hearts, Value::Six)
            ],
            *game.get_dealer_hand()
        );
        assert_eq!(250.0, game.get_player_money(player1)?);
        Ok(())
    }

    #[test]
    fn hit() -> Result<(), GameError> {
        // Test hit action
        let player1 = PlayerID::new_v4();
        let mut game: GameState = GameState::new(vec![player1]);
        game.create_users_hand();
        game.get_mut_deck().append(&mut vec![
            Card::new(CardSuit::Hearts, Value::Ace),
            Card::new(CardSuit::Hearts, Value::Ten),
            Card::new(CardSuit::Hearts, Value::Two),
            Card::new(CardSuit::Hearts, Value::Three),
            Card::new(CardSuit::Hearts, Value::King),
        ]);
        game.action(GameAction::AddMoney(100.0), player1).ok();
        game.action(GameAction::StartingBet(100.0), player1).ok();

        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Ace),
                Card::new(CardSuit::Hearts, Value::Two)
            ],
            *game.get_player_hand(player1)?
        );
        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Ten),
                Card::new(CardSuit::Hearts, Value::Three)
            ],
            *game.get_dealer_hand()
        );

        let test_hit = game.action(GameAction::Hit, player1).ok();
        assert_eq!(
            test_hit,
            Some(vec![ClientEvent::CardRevealed(
                FromPlayer::Player(player1),
                Card::new(CardSuit::Hearts, Value::King)
            ),])
        );

        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Ace),
                Card::new(CardSuit::Hearts, Value::Two),
                Card::new(CardSuit::Hearts, Value::King)
            ],
            *game.get_player_hand(player1)?
        );
        Ok(())
    }

    #[test]
    fn stand() -> Result<(), GameError> {
        // Test stand action
        let player1 = PlayerID::new_v4();
        let player2 = PlayerID::new_v4();
        let mut game: GameState = GameState::new(vec![player1, player2]);
        game.create_users_hand();
        game.get_mut_deck().append(&mut vec![
            Card::new(CardSuit::Hearts, Value::Ace),
            Card::new(CardSuit::Hearts, Value::Ten),
            Card::new(CardSuit::Hearts, Value::Two),
            Card::new(CardSuit::Hearts, Value::Jack),
            Card::new(CardSuit::Hearts, Value::King),
            Card::new(CardSuit::Hearts, Value::Eight),
        ]);
        game.action(GameAction::AddMoney(100.0), player1).ok();
        game.action(GameAction::StartingBet(100.0), player1).ok();
        game.action(GameAction::AddMoney(100.0), player2).ok();
        game.action(GameAction::StartingBet(100.0), player2).ok();
        game.action(GameAction::Stand, player1).ok();
        assert_eq!(Some(player2), game.get_current_player());
        Ok(())
    }

    #[test]
    fn double() -> Result<(), GameError> {
        // Test double action

        let player1 = PlayerID::new_v4();
        let player2 = PlayerID::new_v4();
        let player3 = PlayerID::new_v4();
        let player4 = PlayerID::new_v4();
        let mut game: GameState = GameState::new(vec![player1, player2, player3, player4]);
        game.create_users_hand();
        game.get_mut_deck().append(&mut vec![
            Card::new(CardSuit::Hearts, Value::Ace),
            Card::new(CardSuit::Hearts, Value::Nine),
            Card::new(CardSuit::Hearts, Value::Two),
            Card::new(CardSuit::Hearts, Value::Eight),
            Card::new(CardSuit::Hearts, Value::Ten),
            Card::new(CardSuit::Diamonds, Value::Eight),
            Card::new(CardSuit::Diamonds, Value::Two),
            Card::new(CardSuit::Diamonds, Value::Seven),
            Card::new(CardSuit::Spades, Value::Two),
            Card::new(CardSuit::Spades, Value::Three),
            Card::new(CardSuit::Spades, Value::Five),
            Card::new(CardSuit::Clubs, Value::Five),
            Card::new(CardSuit::Diamonds, Value::Five),
        ]);

        game.action(GameAction::AddMoney(100.0), player1).ok();
        game.action(GameAction::StartingBet(50.0), player1).ok();
        game.action(GameAction::AddMoney(100.0), player2).ok();
        game.action(GameAction::StartingBet(75.0), player2).ok();
        game.action(GameAction::AddMoney(100.0), player3).ok();
        game.action(GameAction::StartingBet(50.0), player3).ok();
        game.action(GameAction::AddMoney(100.0), player4).ok();
        game.action(GameAction::StartingBet(50.0), player4).ok();

        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Ace),
                Card::new(CardSuit::Diamonds, Value::Eight)
            ],
            *game.get_player_hand(player1)?
        );

        // Cannot double if hand is not a sum totaling to 9, 10, or 11
        game.action(GameAction::Double, player1).ok();
        assert_eq!(50.0, game.get_player_money(player1)?);
        assert_eq!(50.0, game.get_player_bet(player1)?);

        // Test return value from Stand
        let test_stand = game.action(GameAction::Stand, player1).ok();
        assert_eq!(test_stand, Some(vec![ClientEvent::PlayerRoundOver]));

        // Cannot double without sufficient money
        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Nine),
                Card::new(CardSuit::Diamonds, Value::Two)
            ],
            *game.get_player_hand(player2)?
        );
        game.action(GameAction::Double, player2).ok();
        assert_eq!(25.0, game.get_player_money(player2)?);
        assert_eq!(75.0, game.get_player_bet(player2)?);
        game.action(GameAction::Stand, player2).ok();

        // Hand sums to 9
        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Two),
                Card::new(CardSuit::Diamonds, Value::Seven)
            ],
            *game.get_player_hand(player3)?
        );

        // Doubling is allowed, hitting and standing is automatically done
        let test_double_3 = game.action(GameAction::Double, player3).ok();
        assert_eq!(
            test_double_3,
            Some(vec![
                ClientEvent::CardRevealed(
                    FromPlayer::Player(player3),
                    Card::new(CardSuit::Spades, Value::Five)
                ),
                ClientEvent::PlayerRoundOver
            ])
        );
        assert_eq!(0.0, game.get_player_money(player3)?);
        assert_eq!(100.0, game.get_player_bet(player3)?);
        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Two),
                Card::new(CardSuit::Diamonds, Value::Seven),
                Card::new(CardSuit::Spades, Value::Five)
            ],
            *game.get_player_hand(player3)?
        );

        // Doubling with sum of 10
        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Eight),
                Card::new(CardSuit::Spades, Value::Two)
            ],
            *game.get_player_hand(player4)?
        );
        game.action(GameAction::Double, player4).ok();
        assert_eq!(0.0, game.get_player_money(player4)?);
        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Eight),
                Card::new(CardSuit::Spades, Value::Two),
                Card::new(CardSuit::Clubs, Value::Five)
            ],
            *game.get_player_hand(player4)?
        );

        // After everyone stands, the dealer draws cards until >= 17
        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Ten),
                Card::new(CardSuit::Spades, Value::Three),
                Card::new(CardSuit::Diamonds, Value::Five)
            ],
            *game.get_dealer_hand()
        );
        Ok(())
    }

    #[test]
    fn bets_returned() -> Result<(), GameError> {
        // Tests if bets are returned
        let player1 = PlayerID::new_v4();
        let player2 = PlayerID::new_v4();
        let mut game: GameState = GameState::new(vec![player1, player2]);
        game.create_users_hand();
        game.get_mut_deck().append(&mut vec![
            Card::new(CardSuit::Hearts, Value::Ace),
            Card::new(CardSuit::Spades, Value::Ten),
            Card::new(CardSuit::Diamonds, Value::Two),
            Card::new(CardSuit::Clubs, Value::Jack),
            Card::new(CardSuit::Clubs, Value::Queen),
            Card::new(CardSuit::Hearts, Value::Eight),
            Card::new(CardSuit::Hearts, Value::Ten),
            Card::new(CardSuit::Hearts, Value::Seven),
        ]);

        // Test return value from AddMoney
        let test_money = game.action(GameAction::AddMoney(100.0), player1).ok();
        assert_eq!(test_money, Some(vec![ClientEvent::Betting(player1, 100.0)]));

        // Test return value from StartingBet
        let test_bet = game.action(GameAction::StartingBet(100.0), player1).ok();
        assert_eq!(test_bet, Some(vec![ClientEvent::Betting(player1, 100.0)]));

        game.action(GameAction::AddMoney(200.0), player2).ok();

        // After everyone bets start_game is run, dealing cards
        game.action(GameAction::StartingBet(50.0), player2).ok();

        // Test dealer hand
        assert_eq!(
            vec![
                Card::new(CardSuit::Diamonds, Value::Two),
                Card::new(CardSuit::Hearts, Value::Eight)
            ],
            *game.get_dealer_hand()
        );

        // Test player1 hand (Natural BlackJack returns x2.5)
        assert_eq!(
            vec![
                Card::new(CardSuit::Hearts, Value::Ace),
                Card::new(CardSuit::Clubs, Value::Jack)
            ],
            *game.get_player_hand(player1)?
        );
        // Bets are mutated at the end of the round
        assert_eq!(100.0, game.get_player_bet(player1)?);

        // Player hits and busts, ending game since no one is next in player_list
        let test_hit = game.action(GameAction::Hit, player2).ok();
        assert_eq!(
            test_hit,
            Some(vec![
                ClientEvent::CardRevealed(
                    FromPlayer::Player(player2),
                    Card::new(CardSuit::Hearts, Value::Ten)
                ),
                ClientEvent::PlayerRoundOver,
                ClientEvent::CardRevealed(
                    FromPlayer::Dealer,
                    Card::new(CardSuit::Hearts, Value::Eight)
                ),
                ClientEvent::CardRevealed(
                    FromPlayer::Dealer,
                    Card::new(CardSuit::Hearts, Value::Seven)
                ),
                ClientEvent::RoundOver
            ])
        );
        assert_eq!(
            vec![
                Card::new(CardSuit::Spades, Value::Ten),
                Card::new(CardSuit::Clubs, Value::Queen),
                Card::new(CardSuit::Hearts, Value::Ten)
            ],
            *game.get_player_hand(player2)?
        );

        // After everyone stands, the dealer draws cards until >= 17
        assert_eq!(
            vec![
                Card::new(CardSuit::Diamonds, Value::Two),
                Card::new(CardSuit::Hearts, Value::Eight),
                Card::new(CardSuit::Hearts, Value::Seven)
            ],
            *game.get_dealer_hand()
        );

        // Bets returned
        assert_eq!(250.0, game.get_player_money(player1)?);
        assert_eq!(150.0, game.get_player_money(player2)?);

        Ok(())
    }
}
