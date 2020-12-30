mod gamestate;
use gamestate::{GameAction, GameState};

fn main() {
    // Testing
    let mut game: GameState = GameState::new();
    let player1 = game.add_user();
    let player2 = game.add_user();

    game.action(GameAction::AddMoney(100), player1).ok();
    game.action(GameAction::StartingBet(50), player1).ok();
    game.start_game();
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
    use gamestate::{ClientEvent, FromPlayer, GameAction, GameError, GameState};

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
        assert_eq!(None, game.get_current_player());
        game.action(GameAction::AddMoney(100), player2).ok();
        game.action(GameAction::StartingBet(100), player2).ok();
        assert_eq!(Some(player1), game.get_current_player());
        Ok(())
    }

    #[test]
    fn hit() -> Result<(), GameError> {
        let mut game: GameState = GameState::new();
        let player1 = game.add_user();
        game.action(GameAction::AddMoney(100), player1).ok();
        game.action(GameAction::StartingBet(100), player1).ok();
        game.dealer_draw();
        game.dealer_draw();
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
        game.dealer_draw();
        game.dealer_draw();
        game.action(GameAction::Hit, player1).ok();
        game.action(GameAction::Hit, player1).ok();
        game.action(GameAction::Stand, player1).ok();
        assert_eq!(Some(player2), game.get_current_player());
        Ok(())
    }

    #[test]
    fn game() -> Result<(), GameError> {
        let mut game: GameState = GameState::new();

        let player1 = game.add_user();
        let player2 = game.add_user();

        // Test return value from AddMoney
        let test_money = game.action(GameAction::AddMoney(100), player1).ok();
        assert_eq!(test_money, Some(vec![ClientEvent::Betting(player1, 100)]));

        // Test return value from StartingBet
        let test_bet = game.action(GameAction::StartingBet(100), player1).ok();
        assert_eq!(test_bet, Some(vec![ClientEvent::Betting(player1, 100)]));

        game.action(GameAction::AddMoney(200), player2).ok();
        game.action(GameAction::StartingBet(50), player2).ok();

        // Test dealer draw and hand
        game.dealer_draw();
        game.dealer_draw();
        assert_eq!(vec![2, 8], game.get_dealer_hand());

        // Test return value from Hit
        let test_hit = game.action(GameAction::Hit, player1).ok();
        assert_eq!(
            test_hit,
            Some(vec![ClientEvent::CardRevealed(
                FromPlayer::Player(player1),
                11
            )])
        );
        game.action(GameAction::Hit, player1).ok();
        assert_eq!(vec![11, 10], *game.get_player_hand(player1)?);

        // Test return value from Stand
        let test_stand = game.action(GameAction::Stand, player1).ok();
        assert_eq!(test_stand, Some(vec![ClientEvent::RoundOver]));

        // Player hits and busts, ending game since no one is next in player_list
        game.action(GameAction::Hit, player2).ok();
        game.action(GameAction::Hit, player2).ok();
        game.action(GameAction::Hit, player2).ok();
        assert_eq!(vec![10, 10, 10], *game.get_player_hand(player2)?);

        // After everyone stands, the dealer draws cards until >= 17
        assert_eq!(vec![2, 8, 7], game.get_dealer_hand());

        // Bets returned
        assert_eq!(200, game.get_player_money(player1)?);
        assert_eq!(150, game.get_player_money(player2)?);

        Ok(())
    }
}
