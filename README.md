# BlackJack

**A backend framework for BlackJack made purely in Rust**

## Architecture
- main.rs
  - Contains the tests showing how the game actions/logic works
- lib/card.rs
  - Contains the implement of the Card type
- lib/gamecoordinator.rs
  - The gamecoordinator controls all of the current games being played and the players playing the games
- lib/gamestate.rs
  - The logic for a single game of BlackJack containing the game actions such as betting, standing, doubling, hitting, dealing cards, and rewarding the bet back to the players
